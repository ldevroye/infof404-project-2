use super::{Job, Task};

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

    pub fn is_feasible(&self) -> bool {
        self.utilisation() <= 1.0 || self.tasks.iter().any(|t| t.period() > t.deadline())
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
