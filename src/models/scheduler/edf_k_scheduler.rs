use crate::*;  // Importing relevant crate components, assumed to be the module that contains TaskSet, Core, etc.
use crate::scheduler::Scheduler;  // Importing the Scheduler trait

/// The `EDFkScheduler` struct holds scheduling parameters and task sets.
pub struct EDFkScheduler {
    task_set: TaskSet,  // The set of tasks to be scheduled
    num_cores: usize,   // The number of available cores for scheduling
    cores: Vec<Core>,   // The cores involved in the scheduling process
    k: usize,           // The number of highest-priority tasks in EDF(k) scheduling
}

impl EDFkScheduler {
    /// Creates a new `EDFkScheduler` with the given task set, number of cores, and k value.
    ///
    /// # Arguments
    ///
    /// * `task_set` - The set of tasks to be scheduled.
    /// * `num_cores` - The number of available cores.
    /// * `k` - The number of highest-priority tasks under EDF(k).
    ///
    /// # Returns
    ///
    /// * A new `EDFkScheduler` instance.
    pub fn new(task_set: TaskSet, num_cores: usize, k: usize) -> Self {
        // Initialize cores with partitioned task sets.
        let cores = (1..=num_cores)
            .map(|id| Core::new_task_set(id as ID, task_set.clone()))
            .collect();
        Self {
            task_set,
            num_cores,
            cores,
            k,
        }
    }

    /// Schedules jobs based on their absolute deadline.
    ///
    /// # Arguments
    ///
    /// * `queue` - A list of jobs along with their core ID.
    ///
    /// # Returns
    ///
    /// * The index of the job with the earliest absolute deadline, or `None` if no job can be scheduled.
    pub fn schedule_jobs(queue: &Vec<(Job, usize)>) -> Option<ID> {
        // Return None if the queue is empty.
        if queue.is_empty() {
            return None;
        }

        let mut smallest = TimeStep::MAX;
        let mut index_to_ret: Option<ID> = None;

        // Iterate over the jobs to find the one with the earliest absolute deadline.
        for (i, (job, id_core)) in queue.iter().enumerate() {
            // Skip jobs already assigned to a core (core ID != 0).
            if *id_core != 0 {
                continue;
            }

            let current_deadline = job.absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = Some(i as ID);
            }
        }

        index_to_ret
    }
}

impl Scheduler for EDFkScheduler {
    /// Checks if the task set is schedulable on the available cores based on EDF(k) criteria.
    ///
    /// # Returns
    ///
    /// * `true` if the task set is schedulable; `false` otherwise.
    fn check_schedulability(&self) -> bool {
        // If k is greater than or equal to task set size, it's not schedulable.
        if self.k >= self.task_set.len() - 1 {
            return false;
        }

        // If any task has full utilization, it's not schedulable.
        if self.task_set.get_task_by_index(self.k).unwrap().utilisation() == 1.0 {
            return false;
        }

        // Check if the task set fits under EDF(k) criteria.
        self.num_cores as f64 >= self.k as f64 + self.task_set.get_task_utilisation(self.k + 1).unwrap() / 
            (1.0 - self.task_set.get_task_utilisation(self.k).unwrap())
    }

    /// Runs the EDF(k) scheduling simulation on the task set.
    ///
    /// # Returns
    ///
    /// * `SchedulingCode::SchedulableSimulated` if the simulation completes successfully.
    /// * `SchedulingCode::UnschedulableSimulated` if the task set is found to be unschedulable during simulation.
    fn run_simulation(&mut self) -> SchedulingCode {
        // If the task set is not schedulable, return early with an unschedulable result.
        if !self.check_schedulability() {
            return SchedulingCode::UnschedulableShortcut;
        }

        let mut queue: Vec<(Job, ID)> = Vec::new();
        let mut result = SchedulingCode::SchedulableSimulated;
        
        // Get the global feasibility interval for the task set.
        let (mut time, max_time) = self.task_set.feasibility_interval_global();
        println!("Interval: [{:?}, {:#?}]", time, max_time);
        let mut threshold = max_time / 100;

        // Set initial time for each core.
        self.cores.iter_mut().for_each(|core| core.set_time(time));

        // Main scheduling loop, running from the start time to the maximum time.
        while time < max_time {
            // Release jobs at the current time step.
            let mut job_to_add = self.task_set.release_jobs(time);

            // Set jobs with task_id < k to have infinite deadline (they are not handled by EDF(k)).
            for job in job_to_add.iter_mut() {
                if job.task_id() < self.k as ID {
                    job.set_deadline_inf();
                }
            }

            // Add jobs to the queue, marking them as unassigned (core ID = 0).
            queue.extend(job_to_add.into_iter().map(|job| (job, 0)));

            // Try to assign jobs to cores.
            for index_core in 0..self.num_cores - 1 {
                let any_available_core = self.cores.iter().any(|core| !core.is_assigned());
                let current_core = self.cores.get_mut(index_core).unwrap();

                // Skip if the queue is empty or the current core's job is already handled.
                if queue.is_empty() || current_core.current_job_is_inf().unwrap_or(false) {
                    break;
                }

                // Schedule a job based on the absolute deadline.
                let result_schedule = EDFkScheduler::schedule_jobs(&queue);
                if result_schedule.is_none() {
                    break;  // No job can be scheduled.
                }

                let elected_index: ID = result_schedule.unwrap();
                let (current_job, _) = {
                    let (job, core) = queue.get_mut(elected_index).unwrap();
                    (job.clone(), *core)
                };

                // If the core is not assigned or the job has a higher priority, schedule it.
                if !current_core.is_assigned() || 
                    (!any_available_core && current_core.is_assigned() && 
                    current_core.current_job_deadline().unwrap() > current_job.absolute_deadline()) {

                    // If the core is already assigned, remove the current job and reset the core ID.
                    if current_core.is_assigned() {
                        current_core.remove(current_job.task_id());
                        if let Some((_, core_ref)) = queue.iter_mut().find(|(_, core_ref)| *core_ref == current_core.id()) {
                            *core_ref = 0;  // Reset the core ID.
                        }
                    }

                    // Assign the new job to the current core.
                    let task_clone = self.task_set.get_task_by_id(current_job.task_id()).unwrap().clone();
                    current_core.add_job(current_job, task_clone);
                }
            }

            // Simulate one step for each core.
            for index_core in 0..self.num_cores - 1 {
                let current_core = self.cores.get_mut(index_core).unwrap();
                let resp = current_core.simulate_step(self.k);
                if resp == CoreValue::Missed {
                    result = SchedulingCode::UnschedulableSimulated;
                    break;
                } else if resp == CoreValue::Complete {
                    // Remove completed job from the queue.
                    queue.retain(|(_, id)| *id != current_core.id());
                } else {  // The job is running, update remaining time.
                    queue.iter_mut()
                        .for_each(|(job, core_id)| if *core_id == current_core.id() {
                            job.set_remaining_time(current_core.get_remaining_time());
                        });
                }
            }

            // Check deadlines for unhandled jobs in the queue.
            for (job, id) in queue.iter() {
                if *id == 0 && job.deadline_missed(time) {
                    result = SchedulingCode::UnschedulableSimulated;
                    println!("Unhandled deadline missed at time {}, task_id {}", time, job.task_id());
                    break;
                }
            }

            // If the result is unschedulable, exit the loop early.
            if result == SchedulingCode::UnschedulableSimulated {
                break;
            }

            time += 1;

            // Print progress if necessary.
            if time > threshold {
                println!("{}% done", (time * 100) / max_time);
                threshold += max_time / 100;
            }
        }

        result  // Return the final result of the simulation.
    }
}