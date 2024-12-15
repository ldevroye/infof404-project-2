use crate::*;  // Importing relevant crate components, assumed to be the module that contains TaskSet, Core, etc.
use crate::scheduler::Scheduler;  // Importing the Scheduler trait

/// The `DMScheduler` struct holds scheduling parameters and task sets.
pub struct DMScheduler {
    task_set: TaskSet,  // The set of tasks to be scheduled
    num_cores: usize,   // The number of available cores for scheduling
    cores: Vec<Core>,   // The cores involved in the scheduling process
}

impl DMScheduler {
    /// Constructor to initialize a new `DMScheduler` with a task set and number of cores.
    pub fn new(task_set: TaskSet, num_cores: usize) -> Self {
        // Initialize cores with partitioned task sets.
        let cores = (1..=num_cores)
            .map(|id| Core::new_task_set(id as ID, task_set.clone()))
            .collect();
        Self {
            task_set,
            num_cores,
            cores,
        }
    }

    /// Schedules jobs based on the deadline order (Earliest Deadline First - EDF).
    pub fn schedule_jobs(queue: &Vec<(Job, ID)>) -> Option<ID> {
        if queue.is_empty() {
            return None;
        }

        let mut best_priority = TimeStep::MAX;  // Initialize with maximum possible priority
        let mut index_to_ret: Option<ID> = None;  // Index of the job to be scheduled

        // Iterate over the queue to find the job with the earliest deadline
        for (i, (_, core_id)) in queue.iter().enumerate() {
            if *core_id != 0 {
                continue;  // Skip jobs already assigned to a core
            }

            // Find the task with the smallest deadline
            let task_priority = queue.iter().map(|(job, _)| job.task_deadline()).min().unwrap();
            if task_priority < best_priority {
                best_priority = task_priority;
                index_to_ret = Some(i as ID);  // Update the index of the job with the earliest deadline
            }
        }

        index_to_ret  // Return the index of the job with the best (earliest) deadline
    }
}

impl Scheduler for DMScheduler {
    /// Runs the simulation of the DM scheduling algorithm.
    fn run_simulation(&mut self) -> SchedulingCode {
        // Sort tasks by their deadline in ascending order (Earliest Deadline First)
        self.task_set.get_tasks_mut().sort_by(|a, b| a.deadline().cmp(&b.deadline()));

        let mut result = SchedulingCode::SchedulableSimulated;  // Default result
        let (mut time, max_time) = self.task_set.feasibility_interval_global();
        let mut thresh_hold = max_time / 100;  // Set threshold for periodic progress reporting
        println!("Time interval: [{}, {}]", time, max_time);

        // Start scheduling simulation over time
        while time < max_time {
            // Release jobs that arrive at the current time
            let job_to_add = self.task_set.release_jobs(time);

            // For DM, priorities are fixed based on the deadline order determined above
            let mut queue: Vec<(Job, ID)> = job_to_add.into_iter().map(|job| (job, 0)).collect();

            // Assign jobs to cores
            for index_core in 0..self.num_cores - 1 {
                let any_available_core = self.cores.iter().any(|core| !core.is_assigned());
                let current_core = self.cores.get_mut(index_core).unwrap();

                // If the queue is empty or the current job is already infinite (i.e., no valid job)
                if queue.is_empty() {
                    break;
                } else if current_core.current_job_is_inf().is_some() {
                    if current_core.current_job_is_inf().unwrap() {
                        continue;
                    }
                }

                // Schedule the job to a core using EDF scheduling
                let result_schedule = DMScheduler::schedule_jobs(&queue);
                if result_schedule.is_none() {
                    break;  // No job can be scheduled to a core
                }

                let elected_index: ID = result_schedule.unwrap();  // Get the selected job index

                // Extract the current job and core ID
                let (_, _) = {
                    let (job, core) = queue.get_mut(elected_index).unwrap();
                    (job.clone(), *core)
                };

                // Get the job and core details
                let (current_job, core_id) = &mut queue[elected_index];

                // Check if we should replace the current job
                if !current_core.is_assigned() ||
                    (!any_available_core &&
                    current_core.is_assigned() &&
                    current_core.current_job_task_deadline().unwrap() > current_job.task_deadline() &&
                    current_core.current_job_deadline().is_some())
                {
                    // Remove the current job if necessary
                    if current_core.is_assigned() {
                        current_core.remove(current_job.task_id());
                    }

                    // Assign the job to the core
                    *core_id = current_core.id();
                    let task_clone = self.task_set.get_task_by_id(current_job.task_id()).unwrap().clone();
                    current_core.add_job(current_job.clone(), task_clone);
                }
            }
        
            // Simulate one time unit for each core
            for index_core in 0..self.num_cores - 1 {
                let current_core = self.cores.get_mut(index_core).unwrap();
                let resp = current_core.simulate_step(1);

                // If a job is missed, mark the scheduling as unschedulable
                if resp == CoreValue::Missed {
                    result = SchedulingCode::UnschedulableSimulated;
                } else if resp == CoreValue::Complete {
                    // Remove completed jobs from the queue
                    queue.retain(|(_, id)| *id != current_core.id());
                }
            }

            // Check deadlines for unhandled jobs
            for (job, id) in &queue {
                if *id == 0 && job.deadline_missed(time) {
                    result = SchedulingCode::UnschedulableSimulated;  // Mark as unschedulable if a deadline is missed
                    break;
                }
            }

            if result == SchedulingCode::UnschedulableSimulated {
                break;  // Exit if the task set is unschedulable
            }

            time += 1;

            // Print progress every 1% of the max time
            if time > thresh_hold {
                println!("{}% done", (time * 100) / max_time);
                thresh_hold += max_time / 100;
            }
        }

        result  // Return the final scheduling result
    }
}