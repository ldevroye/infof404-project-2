
use std::collections::HashMap;
use super::{Task, TaskSet, TimeStep, ID};

#[derive(Debug)]
pub struct Worker {
    id: ID,                             // Unique identifier 
    map_migrations: HashMap<ID, usize>, // map : task -> number migration done        
    utilisation_left: f64,
    current_time: TimeStep,
}

impl Worker {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            map_migrations: HashMap::new(),
            utilisation_left: 1.00,
            current_time: 0,
        }
    }

    /// Add or increment the value of the task_id
    /// 
    /// # Arguments
    /// 
    /// * 'tas'
    pub fn migrate(&mut self, task_id: ID) {
        self.map_migrations.entry(task_id)
        .and_modify(|v| *v += 1)
        .or_insert(1);
    }

    /// Get all the migrations done
    pub fn get_migrations(&self) -> usize {
        self.map_migrations.values().sum()
    }

    pub fn id(&self) -> ID {
        self.id
    }
}