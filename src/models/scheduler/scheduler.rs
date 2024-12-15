use crate::{Job, SchedulingCode, TaskSet, TimeStep, ID};
use crate::constants::Version;
use crate::scheduler::Core;

use std::cmp::Ordering;
use std::process::exit;
use std::{result, thread};
use std::sync::{Arc, Mutex};
use std::thread::current;

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
    /// Creates a new scheduler instance.
    pub fn new(task_set: TaskSet, num_cores: usize, num_threads: usize, heuristic: String, sorting_order: String) -> Self {
        Self {
            task_set,
            version: Version::Partitioned,
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
        if self.task_set.get_task(k).unwrap().utilisation() == 1.0 {
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
        let mut thread_running = 0; // Tracks threads actively running

        let partition = self.partition_tasks();
    
        // Clone task sets for each core
        self.cores = (1..=self.num_cores)
            .map(|id| Core::new_task_set(id as ID, partition[id - 1].clone()))
            .collect();
        println!("Cores : {:#?}", self.cores);
    
        let mut result = SchedulingCode::SchedulableShortcut;
    
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

            /* 
            {
                let mut threads_lock = threads.lock().unwrap();
    
                // Check if any thread has completed
                threads_lock.retain(|handle| {
                    if handle.is_finished() {
                        thread_running -= 1;
                        false // Remove finished thread from the list
                    } else {
                        true // Keep running threads
                    }
                });
            }
    
            if thread_running < thread_number {
                // Clone the Arc for each new thread
                let partition_clone = Arc::clone(&partition);
    
                let handle = thread::spawn(move || {
                    // Lock the partition inside the thread to safely access it
                    let mut partition_lock = partition_clone.lock().unwrap();
                    let taskset = partition_lock.get_mut(processor_done).unwrap();
                    println!("Processor {} is running.", processor_done + 1);
                    simulate(taskset); // Simulate processing of each processor
                    println!("Processor {} is done.", processor_done + 1);
                });
    
                {
                    let mut threads_lock = threads.lock().unwrap();
                    threads_lock.push(handle);
                }
    
                thread_running += 1;
                processor_done += 1;
            } else {
                // If max threads are running, wait a bit
                thread::sleep(std::time::Duration::from_millis(100));
            }
            
        }
    
        // Wait for all threads to complete
        for handle in Arc::try_unwrap(threads).unwrap().into_inner().unwrap() {
            handle.join().unwrap();
        }
        
    
        SchedulingCode::SchedulableSimulated
        */
    }


    /// edf_k job scheduling : gives the job with lowest shown deadline
    pub fn schedule_jobs(queue: &Vec<(Job, usize)>, k: usize) -> Option<ID> {

        // Initialize with the absolute deadline of the first job in the list.
        
        if queue.len() == 0 {
            return None;
        }

        let mut smallest = TimeStep::MAX;
        let mut index_to_ret: Option<ID> = None;

        // Find the job with the earliest deadline
        for i in 0..queue.len() {
            if smallest == TimeStep::MIN {
                break;
            }

            let job = &queue[i];
            let current_deadline = job.absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = Some(i as ID);
            }
        }

        index_to_ret
    }
    
    /// Computes the task set among all the cores using EDF(k), with possible migrations between cores during simulation.
    pub fn compute_edfk(&mut self) -> SchedulingCode {

        if !self.check_edf_k_schedulability() {
            return SchedulingCode::UnschedulableShortcut;
        }

        let k = self.get_k();
        let mut queue: Vec<(Job, ID)> = Vec::new();
        let mut result = SchedulingCode::SchedulableSimulated;

        // Compute the feasibility interval for the task set
        let (mut time, max_time) = self.task_set.feasibility_interval_global();
        println!("interval : [{:?}, {:#?}]", time, max_time);
        let mut threash_hold = max_time / 100;

        // Set the initial time for all cores
        self.cores.iter_mut().for_each(|core| core.set_time(time));

        while time < max_time {
            let mut job_to_add = self.task_set.release_jobs(time);

            // Set the jobs' deadline for task migration
            if !job_to_add.is_empty() {
                for i in 0..job_to_add.len() {
                    job_to_add[i].set_deadline(time);
                }
            }

            // Handle migrations across cores during scheduling
            for (mut task, core_id) in job_to_add {
                let mut assignable_core = self.cores.get_mut(core_id as usize).unwrap();
                if !assignable_core.is_accepted(task) {
                    assignable_core = self.cores.get_mut(core_id as usize).unwrap();
                    if !assignable_core.accepted {
                        continue;
                    }
                    assignable_core.add_task(task);
                } else {
                    assignable_core.add_task(task);
                }
            }

            let mut response_code = SchedulingCode::SchedulableSimulated;

            // Check for task migrations across cores
            for mut job in self.cores.iter_mut() {
                if !job.processing {
                    continue;
                }

                let result_schedule = Scheduler::schedule_jobs_dm(&queue);
                if result_schedule.is_none() { // there is no job not handleled by a core
                    break;
                }

                let elected_index: ID = result_schedule.unwrap();

                // Extract the `current_job` and `core_id` without holding a mutable borrow
                let (current_job, core_id) = {
                    let (job, core) = queue.get_mut(elected_index).unwrap();
                    (job.clone(), *core)
                };

            
                let (current_job, core_id) = &mut queue[elected_index];

                // Check if we should replace current job
                if !current_core.is_assigned() ||
                    (!any_available_core &&
                    current_core.is_assigned() &&
                    current_core.current_job_task_deadline().unwrap() > current_job.task_deadline() &&
                    current_core.current_job_deadline().is_some())
                {
                    if current_core.is_assigned() {
                        current_core.remove(current_job.task_id());
                    }

                    *core_id = current_core.id();
                    let task_clone = self.task_set.get_task_by_id(current_job.task_id()).unwrap().clone();
                    current_core.add_job(current_job.clone(), task_clone);
                }
                
            }
        
            // Simulate one time unit
            for index_core in 0..self.num_cores-1 {
                let current_core = self.cores.get_mut(index_core).unwrap();
                let resp = current_core.simulate_step(1);

                if resp == CoreValue::Missed {
                    result = SchedulingCode::UnschedulableSimulated;
                } else if resp == CoreValue::Commplete {
                    // Remove completed jobs from queue
                    queue.retain(|(_, id)| *id != current_core.id());
                }
            }

            // Check deadlines for unhandled jobs
            for (job, id) in &queue {
                if *id == 0 && job.deadline_missed(time) {
                    result = SchedulingCode::UnschedulableSimulated;
                    break;
                }
            }

            if result == SchedulingCode::UnschedulableSimulated {
                break;
            }

            time += 1;
            if (time > thresh_hold) {
                println!("{}% done", (time*100)/max_time);
                thresh_hold += max_time/100;
            }
        }

        result
    }




    /// Helper function for DM to pick the job with the highest priority (lowest relative deadline task).
    fn schedule_jobs_dm(queue: &Vec<(Job, ID)>) -> Option<ID> {
        if queue.is_empty() {
            return None;
        }

        let mut best_priority = TimeStep::MAX;
        let mut index_to_ret: Option<ID> = None;

        for (i, (job, core_id)) in queue.iter().enumerate() {
            if *core_id != 0 {
                // Already taken by a core
                continue;
            }

            // Get task priority (ie, task deadline)
            let task_priority = queue.iter().map(|(job, _)| job.task_deadline()).min().unwrap();
            if task_priority < best_priority {
                best_priority = task_priority;
                index_to_ret = Some(i as ID);
            }
        }

        index_to_ret
    }


    /// Hub function to chose which version to use
    pub fn test_task_set(&mut self) -> SchedulingCode {
        let result: SchedulingCode;
        match self.version {
            Version::EDFk(value) => {
                result = self.compute_edfk();
            }
            Version::Global => {
                result = self.compute_global();
            }
            Version::Partitioned => {
                result = self.compute_partitionned();
            }
            Version::GlobalDM => {
                result = self.compute_dm();
            }

            _ => {
                eprintln!("Unknow EDF version");
                return SchedulingCode::CannotTell;
            }
        }

        let migrations: usize = self.cores.iter().map(|core| core.get_migrations()).sum();
        println!("Number migrations : {}", migrations);
        return result;
        
    }
}