
use std::collections::HashMap;
use super::{TimeStep, ID};

#[derive(Debug)]
pub struct Worker {
    id: ID,                         // Unique identifier 
    map_task: HashMap<ID, usize>,   // map : task -> number migration done
}

impl Worker {
    pub fn new(id: ID, map_task: HashMap<ID, usize>) -> Self {
        Self {
            id,
            map_task,
        }
    }


    /// Add or increment the value of the task_id
    /// 
    /// # Arguments
    /// 
    /// * 'tas'
    pub fn migrate(&mut self, task_id: ID) {
        self.map_task.entry(task_id)
        .and_modify(|v| *v += 1)
        .or_insert(1);
    }

    /// Get all the migrations done
    pub fn get_migrations(&self) -> usize {
        self.map_task.values().sum()
    }

    pub fn id(&self) -> ID {
        self.id
    }
}