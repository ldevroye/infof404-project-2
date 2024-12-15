use crate::constants::CoreValue;
use crate::{Job, SchedulingCode, TaskSet, TimeStep, ID, Task};

use std::collections::HashMap;

/// Represents a single core in the scheduling system.
#[derive(Debug)]
pub struct Core {
    id: ID,                             // Unique identifier for the core
    map_migrations: HashMap<ID, usize>, // Maps task ID to the number of migrations performed
    task_set: TaskSet,                  // The set of tasks assigned to this core
    queue: Vec<Job>,                    // Queue of jobs for the core
    utilisation_left: f64,              // Remaining utilization of the core
    current_time: TimeStep,             // Current time step of the simulation
    feasibility_interval: Option<TimeStep>, // Feasibility interval for the core
}

impl Core {
    /// Creates a new `Core` instance with default values.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for this core.
    ///
    /// # Returns
    /// A new `Core` instance with an empty task set and default values.
    pub fn new(id: ID) -> Self {
        Self {
            id,
            map_migrations: HashMap::new(),
            task_set: TaskSet::new_empty(),
            queue: Vec::new(),
            utilisation_left: 1.00,
            current_time: 0,
            feasibility_interval: None,
        }
    }

    /// Creates a new `Core` instance with a specific TaskSet.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for this core.
    /// * `task_set` - The set of tasks assigned to the core.
    ///
    /// # Returns
    /// A new `Core` instance with the specified TaskSet.
    pub fn new_task_set(id: ID, task_set: TaskSet) -> Self {
        Self {
            id,
            map_migrations: task_set.iter().map(|task| (task.id(), 0)).collect(),
            utilisation_left: 1.00 - task_set.utilisation(),
            task_set,
            queue: Vec::new(),
            current_time: 0,
            feasibility_interval: None,
        }
    }

    /// Returns whether the core has tasks assigned to it.
    pub fn is_assigned(&self) -> bool {
        !self.task_set.is_empty()
    }

    /// Returns the total remaining time of all jobs in the queue.
    pub fn get_remaining_time(&self) -> TimeStep {
        self.queue.iter().map(|job| job.remaining_time()).sum::<TimeStep>()
    }

    /// Attempts to add a task to the core if it does not exceed the utilization limit.
    ///
    /// # Arguments
    /// * `task` - The task to add to the core.
    ///
    /// # Returns
    /// `true` if the task was successfully added, `false` otherwise.
    pub fn add(&mut self, task: Task) -> bool {
        // Check if the task is already assigned to the core
        if self.task_set.get_task_by_id(task.id()).is_some() {
            return false;
        }

        // Ensure the task does not exceed the core's remaining utilization
        if self.utilisation_left - task.utilisation() < 0.0 {
            return false;
        }

        // Update the core's utilization and migrate the task
        self.utilisation_left -= task.utilisation();
        self.migrate(task.id());
        self.task_set.add_task(task);

        true
    }

    /// Adds a job to the core and assigns a task to it.
    ///
    /// # Arguments
    /// * `job` - The job to add.
    /// * `task` - The task to assign to the job.
    pub fn add_job(&mut self, job: Job, task: Task) {
        self.queue.clear(); // Clear existing jobs in the queue
        self.queue.push(job); // Add the new job to the queue
        self.add(task); // Add the task to the core
    }

    /// Returns the ID of the first task in the task set, if any.
    pub fn get_current_task_id(&self) -> Option<ID> {
        self.task_set.get_first_task_id()
    }

    /// Returns the absolute deadline of the current job, if any.
    pub fn current_job_deadline(&self) -> Option<TimeStep> {
        if self.task_set.is_empty() {
            return None;
        }

        Some(self.queue[0].absolute_deadline())
    }

    /// Checks if the current job's deadline is infinite.
    pub fn current_job_is_inf(&self) -> Option<bool> {
        if self.task_set.is_empty() {
            return None;
        }

        Some(self.queue[0].is_deadline_inf())
    }

    /// Returns the task's deadline for the current job, if any.
    pub fn current_job_task_deadline(&self) -> Option<TimeStep> {
        if self.task_set.is_empty() {
            return None;
        }

        Some(self.queue[0].task_deadline())
    }

    /// Removes a task from the core.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to remove.
    pub fn remove(&mut self, task_id: ID) {
        // Check if the task exists in the core
        if !self.task_set.task_exists(task_id) {
            return;
        }

        // Remove the task from the task set and the job queue
        self.task_set.retain_not(task_id);
        self.queue.retain(|job| job.task_id() != task_id);
        self.migrate(task_id); // Migrate the task after removal
    }

    /// Increments the migration count of a task or initializes it if it does not exist.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to increment the migration count.
    pub fn migrate(&mut self, task_id: ID) {
        self.map_migrations
            .entry(task_id)
            .and_modify(|v| *v += 1)
            .or_insert(1); // Initialize migration count if not present
    }

    /// Returns the total number of migrations performed for all tasks on the core.
    pub fn get_migrations(&self) -> usize {
        self.map_migrations.values().sum()
    }

    /// Returns the ID of the core.
    pub fn id(&self) -> ID {
        self.id
    }

    /// Updates the current time step for the core.
    pub fn set_time(&mut self, new: TimeStep) {
        self.current_time = new;
    }

    /// Schedules the next job in the queue based on EDF (Earliest Deadline First).
    ///
    /// # Arguments
    /// * `k` - The priority level for scheduling (used to filter jobs).
    ///
    /// # Returns
    /// The ID of the scheduled job, if any.
    pub fn schedule(&self, k: usize) -> Option<ID> {
        // Filter jobs with task_id less than `k` and return the one with the smallest task_id
        let smallers: Vec<(ID, ID)> = self.queue.iter()
            .filter(|job| job.task_id() < k as ID)
            .map(|job| (job.task_id(), job.id()))
            .collect();

        if !smallers.is_empty() {
            let (_, job_id) = smallers.iter().min_by_key(|&&(task_id, _)| task_id).unwrap();
            return Some(*job_id);
        }

        // If no job is found, find the job with the earliest absolute deadline
        if self.queue.is_empty() {
            return None;
        }

        let mut smallest = self.queue[0].absolute_deadline();
        let mut index_to_ret: ID = 0;

        for (i, job) in self.queue.iter().enumerate() {
            let current_deadline = job.absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = i as ID;
            }
        }

        Some(index_to_ret)
    }

    /// Tests for possible basic scheduling shortcuts, such as checking feasibility.
    ///
    /// # Returns
    /// An `Option<SchedulingCode>` indicating the result of the test (schedulable or unschedulable).
    pub fn test_shortcuts(&self) -> Option<SchedulingCode> {
        if self.task_set.is_empty() {
            return Some(SchedulingCode::SchedulableShortcut); // No tasks, trivially schedulable
        }

        if !self.task_set.is_feasible() {
            return Some(SchedulingCode::UnschedulableShortcut); // If tasks are not feasible, return unschedulable
        }

        None
    }

    /// Simulates the scheduling for partitioned tasks.
    ///
    /// # Returns
    /// A `SchedulingCode` indicating the result of the simulation (schedulable or unschedulable).
    pub fn simulate_partitionned(&mut self) -> SchedulingCode {
        if let Some(result_shortcut) = self.test_shortcuts() {
            return result_shortcut; // Early return if a shortcut is found
        }

        self.feasibility_interval = Some(self.task_set.feasibility_interval_part().1);

        while self.current_time < self.feasibility_interval.unwrap() {
            // Release new jobs at the current time step
            self.queue.extend(self.task_set.release_jobs(self.current_time));

            // Check for missed deadlines
            if self.queue.iter().any(|job| job.deadline_missed(self.current_time)) {
                return SchedulingCode::UnschedulableSimulated;
            }

            // Schedule the next job if possible
            if let Some(index_elected) = self.schedule(1) {
                if let Some(elected) = self.queue.get_mut(index_elected as usize) {
                    elected.schedule();
                }
            }

            // Remove completed jobs from the queue
            self.queue.retain(|job| !job.is_complete());

            self.current_time += 1;
        }

        SchedulingCode::SchedulableSimulated
    }

    /// Simulates one step of scheduling and checks for missed deadlines or job completion.
    ///
    /// # Arguments
    /// * `k` - The priority level for scheduling (currently unused).
    ///
    /// # Returns
    /// A `CoreValue` indicating the status of the core after the simulation step.
    pub fn simulate_step(&mut self, _: usize) -> CoreValue {
        let mut result = CoreValue::Running;

        // Check if any job has missed its deadline
        if self.queue.iter().any(|job| job.deadline_missed(self.current_time)) {
            result = CoreValue::Missed; // Mark the core as having missed a deadline
        } else {
            // If the queue is not empty, schedule the first job
            if !self.queue.is_empty() {
                let job = self.queue.get_mut(0).unwrap();
                job.schedule();

                // Clear the queue if the job is complete
                if job.is_complete() {
                    self.task_set.get_tasks_mut().clear();
                    self.queue.clear();
                    result = CoreValue::Complete; // Mark the core as complete
                }
            }
        }

        self.current_time += 1; // Increment the time step
        result
    }
}