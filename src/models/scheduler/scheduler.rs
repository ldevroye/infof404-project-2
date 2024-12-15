use crate::{SchedulingCode, TaskSet, Core};
use crate::constants::{Version, ID};

use std::cmp::Ordering;
use std::process::exit;
use std::thread;
use std::sync::{Arc, Mutex};

pub struct Scheduler {
    task_set: TaskSet,
    version: Version,
    num_cores: usize,
    num_threads: usize,
    cores: Vec<Core>,
    heuristic: String,
    sorting_order: String,
}

impl Scheduler {
    /// Creates a new scheduler instance with a specified task set, number of cores, threads, heuristic, and sorting order.
    pub fn new(task_set: TaskSet, num_cores: usize, num_threads: usize, heuristic: String, sorting_order: String) -> Self {
        Self {
            task_set,
            version: Version::PartitionEDF, // Default version
            num_cores,
            num_threads,
            cores: (1..=num_cores).map(|id| Core::new(id as ID)).collect(),
            heuristic,
            sorting_order,
        }
    }

    /// Sets a new version for the scheduler.
    pub fn set_version(&mut self, new_version: Version) {
        self.version = new_version;
    }

    /// Returns the 'k' value of EDF(k), or 1 if the version is not EDF(k).
    pub fn get_k(&self) -> usize {
        match self.version {
            Version::EDFk(value) => value,
            _ => 1, // EDF(1) is just EDF
        }
    }

    /// Checks if the task set is schedulable under global EDF.
    pub fn check_global_edf_schedulability(&self) -> bool {
        let num_cores_f64 = self.num_cores as f64;
        self.task_set.utilisation() <= num_cores_f64 - (num_cores_f64 - 1.0) * self.task_set.max_utilisation()
    }

    /// Checks if the task set is schedulable under EDF(k).
    pub fn check_edf_k_schedulability(&self) -> bool {
        let k = self.get_k();

        // If k is greater than or equal to task set size, it's not schedulable
        if k >= self.task_set.len() - 1 {
            return false;
        }

        // If any task has full utilization, it's not schedulable
        if self.task_set.get_task_by_index(k).unwrap().utilisation() == 1.0 {
            return false;
        }

        // Check if the task set fits under EDF(k)
        self.num_cores as f64 >= k as f64 + self.task_set.get_task_utilisation(k + 1).unwrap() / (1.0 - self.task_set.get_task_utilisation(k).unwrap())
    }

    /// Partitions tasks across processors based on the selected heuristic and sorting order.
    pub fn partition_tasks(&mut self) -> Vec<TaskSet> {
        // Sort tasks based on utilization in the specified order
        if self.sorting_order == "du" {
            self.task_set.get_tasks_mut().sort_by(|a, b| b.utilisation().partial_cmp(&a.utilisation()).unwrap_or(Ordering::Equal));
        } else if self.sorting_order == "iu" {
            self.task_set.get_tasks_mut().sort_by(|a, b| a.utilisation().partial_cmp(&b.utilisation()).unwrap_or(Ordering::Equal));
        } else {
            panic!("Unknown sorting order");
        }

        // Initialize empty partitions for each core
        let mut partitions: Vec<TaskSet> = vec![TaskSet::new_empty(); self.num_cores];
        let num_task = self.task_set.get_tasks_mut().len();

        match self.heuristic.as_str() {
            "ff" => { // First-fit heuristic
                for task in self.task_set.get_tasks_mut().iter() {
                    for task_set in partitions.iter_mut() {
                        if task_set.iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() <= 1.0 {
                            task_set.add_task(task.clone());
                            break;
                        }
                    }
                }
            }
            "nd" => { // Next-fit heuristic
                let mut current_partition = 0;
                for task in self.task_set.get_tasks_mut().iter() {
                    if partitions[current_partition].iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() > 1.0 {
                        current_partition = (current_partition + 1) % self.num_cores;
                    }
                    partitions[current_partition].add_task(task.clone());
                }
            }
            "bf" => { // Best-fit heuristic
                for task in self.task_set.get_tasks_mut().iter() {
                    let mut best_partition: Option<usize> = None;
                    let mut min_slack = f64::MAX;

                    for (i, task_set) in partitions.iter().enumerate() {
                        let slack = 1.0 - task_set.iter().map(|t| t.utilisation()).sum::<f64>();
                        if slack >= task.utilisation() && slack < min_slack {
                            best_partition = Some(i);
                            min_slack = slack;
                        }
                    }

                    if let Some(best) = best_partition {
                        partitions[best].add_task(task.clone());
                    }
                }
            }
            "wf" => { // Worst-fit heuristic
                for task in self.task_set.get_tasks_mut().iter() {
                    let mut worst_partition: Option<usize> = None;
                    let mut max_slack = f64::MIN;

                    for (i, task_set) in partitions.iter().enumerate() {
                        let slack = 1.0 - task_set.iter().map(|t| t.utilisation()).sum::<f64>();
                        if slack >= task.utilisation() && slack > max_slack {
                            worst_partition = Some(i);
                            max_slack = slack;
                        }
                    }

                    if let Some(worst) = worst_partition {
                        partitions[worst].add_task(task.clone());
                    }
                }
            }
            _ => panic!("Unknown heuristic"),
        }

        // Check if the task set can fit into the partitions
        let num_task_computed: usize = partitions.iter().map(|taskset| taskset.len()).sum();
        if num_task > num_task_computed {
            println!("{:#?}", partitions);
            eprintln!("Too many tasks ({:?}, {:?} attached to a processor) for {:?} processors", num_task, num_task_computed, self.num_cores);
            exit(SchedulingCode::UnschedulableShortcut as i32)
        }

        partitions
    }

    /// Main function for partitioned EDF scheduling.
    pub fn compute_partitionned(&mut self) -> SchedulingCode {
        let partition = self.partition_tasks();
    
        // Clone task sets for each core
        self.cores = (1..=self.num_cores)
            .map(|id| Core::new_task_set(id as ID, partition[id - 1].clone()))
            .collect();
        println!("Cores : {:#?}", self.cores);
        
        // Use a shared counter to track completed cores
        let num_core_done = Arc::new(Mutex::new(0));
    
        // Wrap cores in an `Arc<Mutex<_>>` for shared access across threads
        let core_clone = self.cores.clone();
        let cores = Arc::new(Mutex::new(core_clone));
        let num_cores = self.num_cores;
    
        // Shared result state to aggregate scheduling results
        let result_shared = Arc::new(Mutex::new(SchedulingCode::SchedulableShortcut));
    
        // Spawn threads to process cores
        let mut handles = Vec::new();
        for _ in 1..self.num_threads {
            let num_core_done = Arc::clone(&num_core_done);
            let cores = Arc::clone(&cores);
            let result_shared = Arc::clone(&result_shared);
    
            let handle = thread::spawn(move || {
                loop {
                    let mut done = num_core_done.lock().unwrap();
                    if *done >= num_cores {
                        break; // All cores have been processed
                    }
            
                    // Get the next core to process
                    let core_id = *done;
                    *done += 1; // Mark core as being processed
                    drop(done); // Release lock before processing
            
                    let mut cores_guard = cores.lock().unwrap();
                    if let Some(current_core) = cores_guard.get_mut(core_id) {
                        // Directly access the core safely
                        let resp = current_core.simulate_partitionned();
            
                        let mut result_lock = result_shared.lock().unwrap();
                        if !(resp == SchedulingCode::SchedulableShortcut || resp == SchedulingCode::SchedulableSimulated) {
                            println!("Taskset not schedulable");
                            *result_lock = resp;
                        } else if resp == SchedulingCode::SchedulableSimulated {
                            *result_lock = SchedulingCode::SchedulableSimulated;
                        }
                        drop(result_lock); // Release the result lock
                    } else {
                        println!("Error: Could not get core with id {}", core_id);
                    }
                }
            });
    
            handles.push(handle);
        }
    
        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }
    
        // Return the aggregated result
        let final_result = result_shared.lock().unwrap();
        return final_result.clone();
    }

    /// edf_k job scheduling: gives the sequence of the task execution based on EDF(k) policy.
    pub fn edf_k(&mut self) -> SchedulingCode {
        let partition = self.partition_tasks();
        self.cores = (1..=self.num_cores)
            .map(|id| Core::new_task_set(id as ID, partition[id - 1].clone()))
            .collect();
    
        let num_core_done = Arc::new(Mutex::new(0));
        let cores = Arc::new(Mutex::new(self.cores.clone()));
        let num_cores = self.num_cores;
    
        let result_shared = Arc::new(Mutex::new(SchedulingCode::SchedulableShortcut));
    
        let mut handles = Vec::new();
        for _ in 1..self.num_threads {
            let num_core_done = Arc::clone(&num_core_done);
            let cores = Arc::clone(&cores);
            let result_shared = Arc::clone(&result_shared);
    
            let handle = thread::spawn(move || {
                loop {
                    let mut done = num_core_done.lock().unwrap();
                    if *done >= num_cores {
                        break;
                    }
            
                    let core_id = *done;
                    *done += 1;
                    drop(done);
            
                    let mut cores_guard = cores.lock().unwrap();
                    if let Some(current_core) = cores_guard.get_mut(core_id) {
                        let resp = current_core.edf_k_schedule();
                        let mut result_lock = result_shared.lock().unwrap();
                        if !(resp == SchedulingCode::SchedulableShortcut || resp == SchedulingCode::SchedulableSimulated) {
                            println!("Taskset not schedulable");
                            *result_lock = resp;
                        } else if resp == SchedulingCode::SchedulableSimulated {
                            *result_lock = SchedulingCode::SchedulableSimulated;
                        }
                        drop(result_lock);
                    } else {
                        println!("Error: Could not get core with id {}", core_id);
                    }
                }
            });
    
            handles.push(handle);
        }
    
        for handle in handles {
            handle.join().unwrap();
        }
    
        let final_result = result_shared.lock().unwrap();
        return final_result.clone();
    }
}