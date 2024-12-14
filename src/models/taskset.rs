
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

    pub fn is_feasible(&self) -> bool {
        self.utilisation() <= 1 as f64 && self.tasks.iter().all(|t| t.wcet() <= t.deadline())
        
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

    pub fn get_tasks_copy(self) -> Vec<Task> {
        self.tasks
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// retain all the tasks that don't have the 'id'
    pub fn retain(&mut self, id: ID) {
        self.tasks.retain(|task| task.id() != id);
    }

    /// Returns the task at index i (basic case is that index = id)
    pub fn get_task(&self, index: usize) -> Option<&Task> {
        if index >= self.tasks.len() {return None;}

        self.tasks.get(index)
    }

    /// Returns the task that has the id : 'task_id' if it exists
    pub fn get_task_by_id(&self, task_id: ID) -> Option<&Task> {
        self.tasks.iter().find(|&task| task.id() == task_id)
    }

    /// Check if a task with the id 'task_id' exists in the taskset
    pub fn task_exists(&self, task_id: ID) -> bool {
        self.tasks.iter().any(|task| task.id() == task_id)
    }

    pub fn get_task_utilisation(&self, index: usize) -> Option<f64> {
        Some(self.get_task(index).unwrap().utilisation())

    }

    /// Returns the highest utilisation out of all the tasks
    pub fn max_utilisation(&self) -> f64 {
        self.tasks.iter()
        .map(|task| task.utilisation())
        .filter(|&util| !util.is_nan()) // Ignore NaN values
        .fold(f64::NEG_INFINITY, f64::max)
    }


    pub fn checking_schedulability(&self) -> bool {
        false
    }
    pub fn schedulability_proven(&self, _: &TaskSet) -> bool {
        false
    }


    pub fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep) {
        // [Omax, Omax + 2P]

        
        let w_0 = taskset
            .iter()
            .map(|task| task.wcet())
            .sum::<TimeStep>();

        let mut w_k = w_0;

        loop {
            let w_k_next = taskset
                .iter()
                .map(|task| {
                    let ceiling_term = (w_k + task.period() - 1) / task.period();
                    ceiling_term * task.wcet()
                })
                .sum();

            if w_k_next == w_k {
                break (w_0, w_k);
            } else {
                w_k = w_k_next;
            }
        }
    }
}