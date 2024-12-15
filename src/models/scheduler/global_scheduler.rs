use std::sync::{Arc, Mutex};
use std::thread;
use crate::*;  // Importing relevant crate components, assumed to be the module that contains TaskSet, Core, etc.
use crate::scheduler::Scheduler;  // Importing the Scheduler trait

/// The `GlobalEDFScheduler` struct holds scheduling parameters and task sets.
pub struct GlobalEDFScheduler {
    task_set: TaskSet,  // The set of tasks to be scheduled
    num_cores: usize,   // The number of cores available for scheduling
    cores: Vec<Core>,   // The cores involved in the scheduling process
}

impl GlobalEDFScheduler {
    /// Creates a new `GlobalEDFScheduler` with the given task set and number of cores.
    ///
    /// # Arguments
    ///
    /// * `task_set` - The set of tasks to be scheduled.
    /// * `num_cores` - The number of available cores.
    ///
    /// # Returns
    ///
    /// * A `GlobalEDFScheduler` instance with the provided task set and cores.
    pub fn new(task_set: TaskSet, num_cores: usize) -> Self {
        // Initialize the cores with the provided task set, ensuring each core gets its own partitioned task set.
        let cores = (1..=num_cores)
            .map(|id| Core::new_task_set(id as ID, task_set.clone()))  // Clone the task set for each core.
            .collect();  // Collect into a vector of cores.

        Self {
            task_set,   // Store the task set
            num_cores,  // Store the number of cores
            cores,      // Store the list of cores
        }
    }
}

impl Scheduler for GlobalEDFScheduler {
    /// Checks if the task set is schedulable on the available cores based on utilization.
    ///
    /// # Returns
    ///
    /// * `true` if the task set is schedulable; `false` otherwise.
    fn check_schedulability(&self) -> bool {
        // Convert the number of cores to a floating-point value for calculations.
        let num_cores_f64 = self.num_cores as f64;

        // Check if the utilization of the task set is within schedulability limits.
        self.task_set.utilisation() <= num_cores_f64 - (num_cores_f64 - 1.0) * self.task_set.max_utilisation()
    }

    /// Runs the simulation of the task set on the available cores.
    ///
    /// # Returns
    ///
    /// * `SchedulingCode::SchedulableSimulated` if the simulation completes successfully.
    /// * `SchedulingCode::UnschedulableShortcut` if the task set is found to be unschedulable early.
    fn run_simulation(&mut self) -> SchedulingCode {
        // If the task set is not schedulable, return early with an unschedulable result.
        if !self.check_schedulability() {
            return SchedulingCode::UnschedulableShortcut;
        }
        
        // Vector to hold the thread handles.
        let mut threads = vec![];

        // Iterate over the cores and simulate partitioning for each core in parallel.
        for core in self.cores.iter().cloned() {
            // Use Arc and Mutex to safely share the core across threads.
            let core = Arc::new(Mutex::new(core));
            let core_clone = core.clone();

            // Spawn a thread to simulate task execution on this core.
            let handle = thread::spawn(move || {
                let mut core = core_clone.lock().unwrap();  // Lock the core for the simulation.
                core.simulate_partitionned();  // Run the simulation on this core.
            });

            // Store the thread handle for later joining.
            threads.push(handle);
        }

        // Wait for all threads to finish their execution.
        for handle in threads {
            handle.join().unwrap();  // Join each thread and unwrap the result (panic if a thread panics).
        }

        // Return a success result after all simulations have completed.
        SchedulingCode::SchedulableSimulated
    }
}