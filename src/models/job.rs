use crate::TimeStep;

use super::ID;

#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    id: ID,                         // Unique identifier for the job
    task_id: ID,                    // Identifier for the associated task
    task_deadline: TimeStep,        // Deadline for the task
    remaining_time: TimeStep,       // Remaining execution time for the job
    real_absolute_deadline: TimeStep,    // Absolute deadline for the job (including offsets)
    show_absolute_deadline: TimeStep,    // the deadline to show for edf k
}

impl Job {
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
        self.show_absolute_deadline
    }

    pub fn set_deadline_inf(&mut self) {
        self.show_absolute_deadline = TimeStep::MIN;
    }

    pub fn is_deadline_inf(&self) -> bool {
        self.show_absolute_deadline == TimeStep::MIN
    }

    pub fn deadline_missed(&self, t: TimeStep) -> bool {
        self.remaining_time > 0 && t >= self.real_absolute_deadline
    }

    pub fn is_complete(&self) -> bool {
        self.remaining_time == 0
    }

    pub fn schedule(&mut self) {
        self.remaining_time -= 1;
    }
}