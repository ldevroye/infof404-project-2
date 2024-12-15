use crate::constants::{TimeStep, ID};

/// Represents a job in the scheduling system.
/// A job has a unique identifier, associated task ID, deadlines, and remaining execution time.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    id: ID,                         // Unique identifier for the job
    task_id: ID,                    // Identifier for the associated task
    task_deadline: TimeStep,        // Deadline for the associated task
    remaining_time: TimeStep,       // Remaining execution time for the job
    real_absolute_deadline: TimeStep,  // Absolute deadline for the job (including offsets)
    show_absolute_deadline: TimeStep,  // Deadline shown for EDF(k) (Earliest Deadline First)
}

impl Job {
    /// Creates a new `Job` instance.
    ///
    /// # Arguments
    /// * `id` - The job's unique identifier.
    /// * `task_id` - The identifier for the associated task.
    /// * `remaining_time` - The time remaining for the job's execution.
    /// * `task_deadline` - The deadline of the associated task.
    /// * `absolute_deadline` - The absolute deadline of the job (including any offsets).
    ///
    /// # Returns
    /// * `Self` - A new `Job` instance.
    pub fn new(id: ID, task_id: ID, remaining_time: TimeStep, task_deadline: TimeStep, absolute_deadline: TimeStep) -> Self {
        Self {
            id,
            task_id,
            task_deadline,
            remaining_time,
            real_absolute_deadline: absolute_deadline,
            show_absolute_deadline: absolute_deadline,
        }
    }

    // **Getters**: Retrieve the value of various fields.

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn task_id(&self) -> ID {
        self.task_id
    }

    pub fn task_deadline(&self) -> TimeStep {
        self.task_deadline
    }

    pub fn remaining_time(&self) -> TimeStep {
        self.remaining_time
    }

    pub fn set_remaining_time(&mut self, new: TimeStep) {
        self.remaining_time = new;
    }

    pub fn absolute_deadline(&self) -> TimeStep {
        self.show_absolute_deadline
    }

    pub fn real_absolute_deadline(&self) -> TimeStep {
        self.real_absolute_deadline
    }

    // **Methods for deadlines and status checks**

    /// Sets a new absolute deadline for the job (shown for EDF scheduling).
    pub fn set_deadline(&mut self, new: TimeStep) {
        self.show_absolute_deadline = new;
    }

    /// Sets the absolute deadline to infinity (representing an undefined or unbounded deadline).
    pub fn set_deadline_inf(&mut self) {
        self.show_absolute_deadline = TimeStep::MIN; // MIN represents an "infinite" deadline
    }

    /// Checks if the job's deadline is set to infinity.
    ///
    /// # Returns
    /// * `true` if the job's deadline is infinity (`MIN`), otherwise `false`.
    pub fn is_deadline_inf(&self) -> bool {
        self.show_absolute_deadline == TimeStep::MIN
    }

    /// Checks if the job has missed its deadline at a specific time.
    ///
    /// # Arguments
    /// * `t` - The current time step to check against the job's absolute deadline.
    ///
    /// # Returns
    /// * `true` if the job has missed its deadline, otherwise `false`.
    pub fn deadline_missed(&self, t: TimeStep) -> bool {
        self.remaining_time > 0 && t >= self.real_absolute_deadline
    }

    /// Checks if the job is complete (i.e., no remaining time).
    ///
    /// # Returns
    /// * `true` if the job is complete, otherwise `false`.
    pub fn is_complete(&self) -> bool {
        self.remaining_time == 0
    }

    /// Simulates the execution of the job by decrementing the remaining execution time by 1.
    pub fn schedule(&mut self) {
        self.remaining_time -= 1; // Execute the job for one time unit
    }
}