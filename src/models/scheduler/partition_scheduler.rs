use std::cmp::Ordering;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::*;
use crate::scheduler::Scheduler;

/// The `PartitionEDFScheduler` struct holds scheduling parameters and task sets.
pub struct PartitionEDFScheduler {
    task_set: TaskSet,    // The set of tasks to be scheduled
    num_cores: usize,     // The number of available cores
    num_threads: usize,   // The number of threads to simulate cores
    heuristic: Heuristic, // The heuristic used for partitioning tasks
    sorting: Sorting,     // The sorting method for tasks based on utilization
    cores: Vec<Core>,     // Cores to schedule the tasks on
}

impl PartitionEDFScheduler {
    /// Constructor for the `PartitionEDFScheduler`. Initializes a new scheduler with a task set, 
    /// number of cores, number of threads, heuristic, and sorting order.
    pub fn new(
        task_set: TaskSet,
        num_cores: usize,
        num_threads: usize,
        heuristic: Heuristic,
        sorting: Sorting,
    ) -> PartitionEDFScheduler {
        PartitionEDFScheduler {
            task_set,
            num_cores,
            num_threads,
            heuristic,
            sorting,
            cores: (1..=num_cores).map(|id| Core::new(id as ID)).collect(),
        }
    }

    /// Partitions tasks across processors based on the selected heuristic and sorting order.
    pub fn partition_tasks(&mut self) -> Vec<TaskSet> {
        // Sort tasks based on utilization in the specified order.
        self.sort_tasks();

        // Initialize empty partitions for each core.
        let mut partitions: Vec<TaskSet> = vec![TaskSet::new_empty(); self.num_cores];
        let num_task = self.task_set.get_tasks_mut().len();

        // Apply the selected heuristic to partition tasks.
        self.apply_heuristic(&mut partitions);

        // Check if the task set can fit into the partitions.
        let num_task_computed: usize = partitions.iter().map(|taskset| taskset.len()).sum();
        if num_task > num_task_computed {
            eprintln!(
                "Too many tasks ({:?} tasks computed) for {:?} processors",
                num_task, self.num_cores
            );
            exit(SchedulingCode::UnschedulableShortcut as i32);
        }

        partitions
    }

    /// Sorts the tasks based on the selected sorting order.
    fn sort_tasks(&mut self) {
        match self.sorting {
            Sorting::DecreasingUtilization => {
                self.task_set.get_tasks_mut().sort_by(|a, b| {
                    b.utilisation()
                        .partial_cmp(&a.utilisation())
                        .unwrap_or(Ordering::Equal)
                });
            }
            Sorting::IncreasingUtilization => {
                self.task_set.get_tasks_mut().sort_by(|a, b| {
                    a.utilisation()
                        .partial_cmp(&b.utilisation())
                        .unwrap_or(Ordering::Equal)
                });
            }
        }
    }

    /// Applies the selected heuristic to partition the tasks across cores.
    fn apply_heuristic(&mut self, partitions: &mut Vec<TaskSet>) {
        match self.heuristic {
            Heuristic::FirstFit => self.first_fit(partitions),
            Heuristic::NextFit => self.next_fit(partitions),
            Heuristic::BestFit => self.best_fit(partitions),
            Heuristic::WorstFit => self.worst_fit(partitions),
        }
    }

    // Heuristic methods for task partitioning:

    /// First-fit heuristic: Place tasks in the first core that has enough slack to fit the task.
    fn first_fit(&self, partitions: &mut Vec<TaskSet>) {
        for task in self.task_set.get_tasks().iter() {
            for task_set in partitions.iter_mut() {
                if task_set.iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() <= 1.0 {
                    task_set.add_task(task.clone());
                    break;
                }
            }
        }
    }

    /// Next-fit heuristic: Place tasks in the current core until it runs out of slack, then move to the next core.
    fn next_fit(&self, partitions: &mut Vec<TaskSet>) {
        let mut current_partition = 0;
        for task in self.task_set.get_tasks().iter() {
            if partitions[current_partition].iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() > 1.0 {
                current_partition = (current_partition + 1) % self.num_cores;
            }
            partitions[current_partition].add_task(task.clone());
        }
    }

    /// Best-fit heuristic: Place tasks in the core with the least slack that can accommodate the task.
    fn best_fit(&self, partitions: &mut Vec<TaskSet>) {
        for task in self.task_set.get_tasks().iter() {
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

    /// Worst-fit heuristic: Place tasks in the core with the most slack.
    fn worst_fit(&self, partitions: &mut Vec<TaskSet>) {
        for task in self.task_set.get_tasks().iter() {
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
}

impl Scheduler for PartitionEDFScheduler {
    /// Runs the simulation of the partitioned EDF scheduler. Tasks are partitioned, assigned to cores, 
    /// and simulated across multiple threads.
    fn run_simulation(&mut self) -> SchedulingCode {
        // Partition the tasks across cores.
        let partition = self.partition_tasks();

        // Initialize cores with assigned task sets
        self.cores = (1..=self.num_cores)
            .map(|id| Core::new_task_set(id as ID, partition[id - 1].clone()))
            .collect();

        // Print the initialized cores for debugging
        println!("Cores: {:#?}", self.cores);

        // Shared state to track the number of completed cores and the result of the scheduling simulation.
        let num_core_done = Arc::new(Mutex::new(0));
        let cores = Arc::new(Mutex::new(self.cores.clone()));
        let result_shared = Arc::new(Mutex::new(SchedulingCode::SchedulableShortcut));

        // Spawn threads to process the scheduling of cores
        let mut handles = Vec::new();
        for _ in 1..self.num_threads {
            let num_core_done = Arc::clone(&num_core_done);
            let cores = Arc::clone(&cores);
            let result_shared = Arc::clone(&result_shared);
            let num_cores = self.num_cores;

            // Spawn a thread for processing each core
            let handle = thread::spawn(move || {
                loop {
                    let mut done = num_core_done.lock().unwrap();
                    if *done >= num_cores {
                        break; // All cores have been processed
                    }

                    // Get the next core to process
                    let core_id = *done;
                    *done += 1;
                    drop(done); // Release the lock before processing the core

                    let mut cores_guard = cores.lock().unwrap();
                    if let Some(current_core) = cores_guard.get_mut(core_id) {
                        // Simulate the task processing for this core
                        let resp = current_core.simulate_partitionned();

                        // Update the shared result based on the scheduling response
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

        // Return the final scheduling result
        let final_result = result_shared.lock().unwrap();
        final_result.clone()
    }
}