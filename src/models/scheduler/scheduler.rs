use crate::{SchedulingCode, TaskSet};

/// The trait defining common scheduling behavior
pub trait Scheduler {
    fn partition_tasks(&mut self) -> Vec<TaskSet>;
    fn is_schedulable(&mut self) -> SchedulingCode;
}