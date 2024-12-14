use crate::constants::EDFVersion;
use crate::{Job, Partition, SchedulingCode, TaskSet, TimeStep, ID};
use crate::scheduler::{Scheduler};
use std::thread;
use std::sync::{Arc, Mutex};

use std::collections::HashMap;

#[derive(Debug)]
pub struct Core {
    id: ID,                             // Unique identifier 
    map_migrations: HashMap<ID, usize>, // map : task -> number migration done        
    task_set: TaskSet,
    utilisation_left: f64,
    current_time: TimeStep,
}

impl Core {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            map_migrations: HashMap::new(),
            task_set: TaskSet::new_empty(),
            utilisation_left: 1.00,
            current_time: 0,
        }
    }

    pub fn new_task_set(id: ID, task_set:TaskSet) -> Self {
        Self {
            id,
            map_migrations: HashMap::new(),
            task_set,
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

    pub fn schedule<'a>(&'a self, jobs: &'a mut Vec<Job>) -> Option<ID> {
        if jobs.is_empty() {
            return None;
        }

        // Initialize with the absolute deadline of the first job in the list.
        let mut smallest = jobs[0].absolute_deadline();
        let mut index_to_ret: ID = 0;

        // Iterate over the jobs to find the one with the earliest absolute deadline.
        let jobs_size = jobs.len();
        for i in 0..jobs_size {
            let current_deadline = jobs[i].absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = i as ID;
            }
        }

        Some(index_to_ret)
    }

    pub fn simulate(&mut self) -> SchedulingCode {

        if self.task_set.is_empty() {
            return SchedulingCode::SchedulableShortcut;
        }

        
        if !self.task_set.is_feasible() {
            return SchedulingCode::UnschedulableShortcut
        }

        
        if self.task_set.checking_schedulability() { // TODO check ?
            if self.task_set.schedulability_proven(&self.task_set) {
                return SchedulingCode::SchedulableShortcut;
            } else {
                return SchedulingCode::UnschedulableShortcut;
            }
        }
        
        let mut queue: Vec<Job> = Vec::new();
        let feasibility_interval = self.task_set.feasibility_interval(&self.task_set).1;
        
        for t in 0..feasibility_interval {
            // Try to release new jobs at time `t`
            queue.extend(self.task_set.release_jobs(t));
            
            // Check for missed deadlines
            if queue.iter().any(|job| job.deadline_missed(t)) {
                println!("time {:?}", t);
                return SchedulingCode::UnschedulableSimulated;
            }

            // Clone the job to be scheduled to avoid multiple mutable borrows
            if let Some(index_elected) = self.schedule(&mut queue){
                if let Some(elected) = queue.get_mut(index_elected as usize) {
                    elected.schedule(1);
                }
            }
        

            // Filter out completed jobs
            queue = queue.into_iter().filter(|job| !job.is_complete()).collect();
        }
        SchedulingCode::SchedulableSimulated
    }
}

    