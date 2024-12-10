use crate::TimeStep;

use super::{Task, ID};

#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    id: ID,                         // Unique identifier for the job
    task_id: ID,                    // Identifier for the associated task
    task_deadline: TimeStep,        // Deadline for the task
    remaining_time: TimeStep,       // Remaining execution time for the job
    absolute_deadline: TimeStep,    // Absolute deadline for the job (including offsets)
}

impl Job {
    pub fn new(id: ID, task_id: ID, remaining_time: TimeStep, task_deadline: TimeStep, absolute_deadline: TimeStep) -> Self {
        Self {
            id,
            task_id,
            task_deadline,
            remaining_time,
            absolute_deadline,
        }
    }

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

    pub fn absolute_deadline(&self) -> TimeStep {
        self.absolute_deadline
    }

    pub fn deadline_missed(&self, t: TimeStep) -> bool {
        self.remaining_time > 0 && t >= self.absolute_deadline
    }

    pub fn is_complete(&self) -> bool {
        self.remaining_time == 0
    }

    pub fn schedule(&mut self, n_steps: TimeStep) {
        self.remaining_time -= n_steps;
    }
}
