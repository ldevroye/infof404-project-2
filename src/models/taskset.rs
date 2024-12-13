
use super::{Job, Task, TimeStep, ID};

#[derive(Debug, Clone)]
pub struct TaskSet {
    tasks: Vec<Task>,
}

impl TaskSet {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    pub fn new_empty() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn utilisation(&self) -> f64 {
        self.tasks.iter().map(|t| t.utilisation()).sum()
    }

    pub fn is_feasible(&self, processor_num: ID) -> bool {
        self.utilisation() <= processor_num as f64 && self.tasks.iter().all(|t| t.wcet() <= t.deadline())
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn release_jobs(&mut self, current_time: TimeStep) -> Vec<Job> {
        self.tasks
            .iter_mut()
            .filter_map(|t| t.spawn_job(current_time))
            .collect()
    }

    pub fn iter(&self) -> std::slice::Iter<Task> {
        self.tasks.iter()
    }

    /// Adds a new task to the task set.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to add.
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

    /// Returns a mutable reference to the vector of tasks in the task set.
    pub fn get_tasks_mut(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    /// Returns the task at index i
    pub fn get_task(&mut self, index: usize) -> Option<&Task> {
        if index >= self.tasks.len() {return None;}

        self.tasks.get(index)
    }
}