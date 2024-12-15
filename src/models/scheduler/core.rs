use crate::constants::CoreValue;
use crate::{Job, SchedulingCode, TaskSet, TimeStep, ID, Task};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Core {
    id: ID,                             // Unique identifier for the core
    map_migrations: HashMap<ID, usize>, // Maps task ID to the number of migrations performed
    task_set: TaskSet,                  // The set of tasks assigned to this core
    queue: Vec<Job>,                    // Job queue for the core
    utilisation_left: f64,              // Remaining utilization of the core
    current_time: TimeStep,             // Current time step of the simulation
    feasibility_interval: Option<TimeStep>, // Feasibility interval for the core
}

impl Core {
    /// Creates a new Core instance.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for this core.
    ///
    /// # Returns
    /// A new `Core` instance.
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

    /// Creates a new Core instance with a specific TaskSet.
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

    /// Tries to add a task to the core if it does not exceed the utilization limit.
    ///
    /// # Arguments
    /// * `task` - The task to add to the core.
    ///
    /// # Returns
    /// `true` if the task was successfully added, `false` otherwise.
    pub fn add(&mut self, task: Task) -> bool {
        if self.task_set.get_task_by_id(task.id()).is_some() {
            return false; // Task already assigned to the core
        }

        if self.utilisation_left - task.utilisation() < 0.0 {
            return false; // Adding the task would exceed utilization
        }

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
        self.queue.clear();
        self.queue.push(job);
        self.add(task);
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

    /// Removes a task from the core by its task ID.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to remove.
    pub fn remove(&mut self, task_id: ID) {
        if !self.task_set.task_exists(task_id) {
            return;
        }

        self.task_set.retain_not(task_id);
        self.queue.retain(|job| job.task_id() != task_id);
        self.migrate(task_id);
    }

    /// Increments (or sets to 1 if non-existent) the migration count of a task.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task whose migration count should be updated.
    pub fn migrate(&mut self, task_id: ID) {
        self.map_migrations
            .entry(task_id)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    /// Returns the total number of migrations that have been performed.
    pub fn get_migrations(&self) -> usize {
        self.map_migrations.values().sum()
    }

    /// Returns the ID of the core.
    pub fn id(&self) -> ID {
        self.id
    }

    /// Schedules the next job based on the specified `k` value.
    ///
    /// # Arguments
    /// * `k` - The `k` value for EDF scheduling (1 for normal EDF).
    ///
    /// # Returns
    /// An `Option<ID>` representing the ID of the next job to schedule.
    pub fn schedule(&self, k: usize) -> Option<ID> {
        // Create a vector of (task_id, job_id) for jobs with task_id less than k
        let smallers: Vec<(ID, ID)> = self.queue.iter()
            .filter(|job| job.task_id() < k as ID)
            .map(|job| (job.task_id(), job.id()))
            .collect();

        // Return the smallest task_id if it's below k
        if !smallers.is_empty() {
            let (_, job_id) = smallers.iter().min_by_key(|&&(task_id, _)| task_id).unwrap();
            return Some(*job_id);
        }

        if self.queue.is_empty() {
            return None;
        }

        // Initialize with the absolute deadline of the first job in the list
        let mut smallest = self.queue[0].absolute_deadline();
        let mut index_to_ret: ID = 0;

        // Find the job with the earliest absolute deadline
        for (i, job) in self.queue.iter().enumerate() {
            let current_deadline = job.absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = i as ID;
            }
        }

        Some(index_to_ret)
    }

    /// Tests for possible basic shortcuts (e.g., unschedulable or schedulable shortcut).
    ///
    /// # Returns
    /// An `Option<SchedulingCode>` indicating the result of the test.
    pub fn test_shortcuts(&self) -> Option<SchedulingCode> {
        if self.task_set.is_empty() {
            return Some(SchedulingCode::SchedulableShortcut);
        }

        if !self.task_set.is_feasible() {
            return Some(SchedulingCode::UnschedulableShortcut);
        }

        None
    }

    /// Sets the feasibility interval for the core based on the task set.
    pub fn set_feasibility_interval(&mut self) {
        self.feasibility_interval = Some(self.task_set.feasibility_interval().1);
    }

    /// Simulates the core's behavior with partitioned tasks.
    pub fn simulate_partitionned(&mut self) -> SchedulingCode {
        if let Some(result_shortcut) = self.test_shortcuts() {
            return result_shortcut;
        }

        self.set_feasibility_interval();

        while self.current_time < self.feasibility_interval.unwrap() {
            // Try to release new jobs at the current time step
            self.queue.extend(self.task_set.release_jobs(self.current_time));

            // Check for missed deadlines
            if self.queue.iter().any(|job| job.deadline_missed(self.current_time)) {
                return SchedulingCode::UnschedulableSimulated;
            }

            // Schedule the next job
            if let Some(index_elected) = self.schedule(1) {
                if let Some(elected) = self.queue.get_mut(index_elected as usize) {
                    elected.schedule();
                }
            }

            // Filter out completed jobs
            self.queue.retain(|job| !job.is_complete());

            self.current_time += 1;
        }

        SchedulingCode::SchedulableSimulated
    }

    /// Simulates one step at a time, checking for missed deadlines and job completions.
    ///
    /// # Arguments
    /// * `k` - The `k` value for EDF scheduling.
    ///
    /// # Returns
    /// A `CoreValue` indicating the status of the core after the step.
    pub fn simulate_step(&mut self, _: usize) -> CoreValue {
        let mut result = CoreValue::Running;

        // Check for missed deadlines
        if self.queue.iter().any(|job| job.deadline_missed(self.current_time)) {
            result = CoreValue::Missed;
        } else {
            if !self.queue.is_empty() {
                let job = self.queue.get_mut(0).unwrap();
                job.schedule();

                // Filter out completed jobs
                if job.is_complete() {
                    self.task_set.get_tasks_mut().clear();
                    self.queue.clear();
                    result = CoreValue::Complete;
                }
            }
        }

        self.current_time += 1;
        result
    }
}