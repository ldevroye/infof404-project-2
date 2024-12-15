use crate::*;

/// A set of tasks in the scheduling system.
#[derive(Debug, Clone)]
pub struct TaskSet {
    tasks: Vec<Task>, // A vector holding the tasks in the task set
}

impl TaskSet {
    /// Creates a new `TaskSet` with a list of tasks.
    /// 
    /// # Arguments
    /// * `tasks` - A vector of tasks to initialize the task set.
    /// 
    /// # Returns
    /// * `Self` - A new `TaskSet` instance.
    pub fn new(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    /// Creates a new empty `TaskSet`.
    /// 
    /// # Returns
    /// * `Self` - A new empty `TaskSet`.
    pub fn new_empty() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Calculates the total utilization of all tasks in the task set.
    /// 
    /// # Returns
    /// * `f64` - The total utilization of the task set.
    pub fn utilisation(&self) -> f64 {
        self.tasks.iter().map(|t| t.utilisation()).sum()
    }

    /// Checks if the task set is feasible based on its utilization and WCET/deadline constraints.
    /// 
    /// # Returns
    /// * `bool` - `true` if the task set is feasible, `false` otherwise.
    pub fn is_feasible(&self) -> bool {
        self.utilisation() <= 1.0 && self.tasks.iter().all(|t| t.wcet() <= t.deadline())
    }

    /// Checks if the task set is empty.
    /// 
    /// # Returns
    /// * `bool` - `true` if the task set has no tasks, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Gets the ID of the first task in the task set.
    /// 
    /// # Returns
    /// * `Option<ID>` - The ID of the first task if the task set is not empty, `None` otherwise.
    pub fn get_first_task_id(&self) -> Option<ID> {
        if self.is_empty() {
            return None;
        }
        Some(self.tasks[0].id())
    }

    /// Attempts to release jobs for tasks at the current time.
    /// 
    /// # Arguments
    /// * `current_time` - The current time step to check for job releases.
    /// 
    /// # Returns
    /// * `Vec<Job>` - A vector of jobs that have been released at the current time.
    pub fn release_jobs(&mut self, current_time: TimeStep) -> Vec<Job> {
        self.tasks
            .iter_mut()
            .filter_map(|t| t.spawn_job(current_time))
            .collect()
    }

    /// Returns an iterator over the tasks in the task set.
    /// 
    /// # Returns
    /// * `std::slice::Iter<Task>` - An iterator over the tasks in the task set.
    pub fn iter(&self) -> std::slice::Iter<Task> {
        self.tasks.iter()
    }

    /// Adds a new task to the task set.
    /// 
    /// # Arguments
    /// * `task` - The task to add to the task set.
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

    /// Returns a mutable reference to the vector of tasks.
    /// 
    /// # Returns
    /// * `&mut Vec<Task>` - A mutable reference to the task vector.
    pub fn get_tasks_mut(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    /// Returns an immutable reference to the vector of tasks.
    /// 
    /// # Returns
    /// * `&Vec<Task>` - An immutable reference to the task vector.
    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    /// Returns a copy of the vector of tasks.
    /// 
    /// # Returns
    /// * `Vec<Task>` - A copy of the task vector.
    pub fn get_tasks_copy(self) -> Vec<Task> {
        self.tasks
    }

    /// Returns the number of tasks in the task set.
    /// 
    /// # Returns
    /// * `usize` - The number of tasks in the task set.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Retains only tasks that do not have the specified ID.
    /// 
    /// # Arguments
    /// * `id` - The ID to retain tasks without.
    pub fn retain_not(&mut self, id: ID) {
        self.tasks.retain(|task| task.id() != id);
    }

    /// Returns the task at the specified index.
    /// 
    /// # Arguments
    /// * `index` - The index of the task to retrieve.
    /// 
    /// # Returns
    /// * `Option<&Task>` - The task at the specified index, or `None` if the index is out of bounds.
    pub fn get_task_by_index(&self, index: usize) -> Option<&Task> {
        if index >= self.tasks.len() {
            return None;
        }
        self.tasks.get(index)
    }

    /// Returns the task with the specified task ID, if it exists.
    /// 
    /// # Arguments
    /// * `task_id` - The ID of the task to retrieve.
    /// 
    /// # Returns
    /// * `Option<&Task>` - The task with the specified ID, or `None` if it does not exist.
    pub fn get_task_by_id(&self, task_id: ID) -> Option<&Task> {
        self.tasks.iter().find(|&task| task.id() == task_id)
    }

    /// Checks if a task with the specified ID exists in the task set.
    /// 
    /// # Arguments
    /// * `task_id` - The ID of the task to check for.
    /// 
    /// # Returns
    /// * `bool` - `true` if the task exists, `false` otherwise.
    pub fn task_exists(&self, task_id: ID) -> bool {
        self.tasks.iter().any(|task| task.id() == task_id)
    }

    /// Returns the utilization of the task at the specified index.
    /// 
    /// # Arguments
    /// * `index` - The index of the task to check.
    /// 
    /// # Returns
    /// * `Option<f64>` - The utilization of the task at the specified index, or `None` if the task does not exist.
    pub fn get_task_utilisation(&self, index: usize) -> Option<f64> {
        Some(self.get_task_by_index(index).unwrap().utilisation())
    }

    /// Returns the highest utilization out of all the tasks in the task set.
    /// 
    /// # Returns
    /// * `f64` - The highest task utilization.
    pub fn max_utilisation(&self) -> f64 {
        self.tasks.iter()
            .map(|task| task.utilisation())
            .filter(|&util| !util.is_nan()) // Ignore NaN values
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Computes the global feasibility interval based on the task set's offset and periods.
    /// 
    /// # Returns
    /// * `(TimeStep, TimeStep)` - The feasibility interval.
    pub fn feasibility_interval_global(&self) -> (TimeStep, TimeStep) {
        // Feasibility interval: [0, Omax + 2 * LCM(periods)]
        
        let o_max = self.tasks.iter().map(|task| task.offset()).max().unwrap();
        let vec_period: Vec<TimeStep> = self.tasks.iter().map(|task| task.period()).collect();
        let p = multiple_lcm(vec_period);

        // Return the feasibility interval based on the offset and periods
        (0, o_max + (2 * p))
    }

    /// Computes the feasibility interval for a part of the task set based on WCET and periods.
    /// 
    /// # Returns
    /// * `(TimeStep, TimeStep)` - The feasibility interval for the part of the task set.
    pub fn feasibility_interval_part(&self) -> (TimeStep, TimeStep) {
        let w_0 = self.tasks.iter().map(|task| task.wcet()).sum::<TimeStep>();

        let mut w_k = w_0;

        loop {
            let w_k_next = self.tasks.iter()
                .map(|task| {
                    let ceiling_term = (w_k + task.period() - 1) / task.period();
                    ceiling_term * task.wcet()
                })
                .sum();

            // If the WCET sums converge, break the loop
            if w_k_next == w_k {
                return (w_0, w_k);
            } else {
                w_k = w_k_next;
            }
        }
    }
}