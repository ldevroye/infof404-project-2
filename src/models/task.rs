use crate::utils::constants::{TimeStep, ID};
use crate::models::job::Job;

/// Struct representing a task in the scheduling system.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    id: ID,             // Unique identifier for the task
    offset: TimeStep,   // O_i : Offset - Time at which the task first becomes ready
    wcet: TimeStep,     // C_i : Worst-case execution time - Time required for the task to complete
    deadline: TimeStep, // D_i : Relative deadline - The time by which the task must complete
    period: TimeStep,   // T_i : Period - The time interval between successive releases of the task
    jobs_released: ID,  // Count of jobs released for the task
}

impl Task {
    /// Creates a new `Task` instance.
    /// 
    /// # Arguments
    /// * `id` - The unique identifier of the task.
    /// * `offset` - The time at which the task first becomes ready.
    /// * `wcet` - The worst-case execution time for the task.
    /// * `deadline` - The relative deadline by which the task must be completed.
    /// * `period` - The time interval between successive task releases.
    /// 
    /// # Returns
    /// * `Self` - A new `Task` instance.
    pub fn new(
        id: ID,
        offset: TimeStep,
        wcet: TimeStep,
        deadline: TimeStep,
        period: TimeStep,
    ) -> Self {
        Self {
            id,
            offset,
            wcet,
            deadline,
            period,
            jobs_released: 0,
        }
    }

    // Getter methods for the task properties

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn offset(&self) -> TimeStep {
        self.offset
    }

    pub fn wcet(&self) -> TimeStep {
        self.wcet
    }

    pub fn deadline(&self) -> TimeStep {
        // Deadline includes the offset
        self.deadline + self.offset
    }

    pub fn period(&self) -> TimeStep {
        // Period includes the offset
        self.period + self.offset
    }

    /// Calculates the task's utilization.
    /// 
    /// # Returns
    /// * `f64` - The utilization of the task (WCET / Period).
    pub fn utilisation(&self) -> f64 {
        self.wcet as f64 / self.period as f64
    }

    /// Spawns a new job for the task if it's time to release a new job.
    /// 
    /// # Arguments
    /// * `current_time` - The current time step to check if a job can be released.
    /// 
    /// # Returns
    /// * `Option<Job>` - A job is returned if the task is ready to release a job, `None` otherwise.
    pub fn spawn_job(&mut self, current_time: TimeStep) -> Option<Job> {
        // If the task has not yet been released (before its offset), no job is spawned
        if current_time < self.offset {
            return None;
        }

        // If the current time is not a multiple of the task's period, it's not time for a release
        if (current_time - self.offset) % self.period != 0 {
            return None;
        }
        
        // Increment the number of jobs released for this task
        self.jobs_released += 1;
        
        // Create and return a new job with the appropriate ID and deadlines
        Some(Job::new(
            self.jobs_released + 1, // Start job ID from 1 (not 0)
            self.id,                // Task ID
            self.wcet,              // Task WCET
            self.deadline(),        // Task deadline
            self.deadline() + current_time, // Absolute deadline based on current time
        ))
    }
}