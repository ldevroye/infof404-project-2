use crate::task;

use super::{Job, Task};

#[derive(Debug)]
pub struct TaskSet {
    tasks: Vec<Task>,
}

impl TaskSet {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    pub fn utilisation(&self) -> f64 {
        self.tasks.iter().map(|t| t.utilisation()).sum()
    }

    pub fn is_feasible(&self, num_processor: u32) -> bool {
        self.utilisation() <= num_processor as f64 && self.tasks.iter().all(|t| t.wcet() <= t.deadline())
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn release_jobs(&mut self, current_time: u32) -> Vec<Job> {
        self.tasks
            .iter_mut()
            .filter_map(|t| t.spawn_job(current_time))
            .collect()
    }

    pub fn iter(&self) -> std::slice::Iter<Task> {
        self.tasks.iter()
    }
}
