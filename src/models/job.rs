use crate::TimeStep;

use super::Task;

#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    id: u32,
    task: Task,
    deadline: TimeStep,         // Absolute deadline for the job.
    remaining_time: TimeStep,   // Remaining execution time for the job.
}

impl Job {
    pub fn new(id: u32, task: Task, deadline: TimeStep) -> Self {
        Self {
            id,
            remaining_time: task.wcet(),
            task,
            deadline,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn task(&self) -> &Task {
        &self.task
    }

    pub fn deadline(&self) -> TimeStep {
        self.deadline
    }

    pub fn remaining_time(&self) -> u32 {
        self.remaining_time
    }

    pub fn deadline_missed(&self, t: TimeStep) -> bool {
        self.remaining_time > 0 && t > self.deadline
    }

    pub fn is_complete(&self) -> bool {
        self.remaining_time == 0
    }

    pub fn schedule(&mut self, n_steps: TimeStep) {
        self.remaining_time -= n_steps;
    }
}
