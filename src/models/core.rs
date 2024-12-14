use crate::constants::EDFVersion;
use crate::{Job, Partition, SchedulingCode, TaskSet, TimeStep, ID};
use crate::scheduler::{Scheduler};
use std::ops::Add;
use std::{task, thread, usize};
use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use super::Task;

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
            map_migrations: task_set.iter().map(|task| (task.id(), 0)).collect(), // put a 0 into each key
            utilisation_left: 1.00 - task_set.utilisation(),
            task_set,
            current_time: 0,
        }
    }

    /// Add as task to the core
    pub fn add(&mut self, task: Task) {
        self.migrate(task.id());
        self.task_set.add_task(task);
        
    }

    /// Remove
    pub fn remove(&mut self, task_id: ID) {
        if !self.task_set.task_exists(task_id) {return;}

        self.task_set.retain(task_id);
        self.migrate(task_id);
    }

    /// increment (or set to 1 if non existent) the number of moves of a task
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

    pub fn schedule(&self, jobs: &mut Vec<Job>, k: usize) -> Option<ID> {
        // create a vector of (task_id, job_id) of all the jobs smaller than k
        let smallers: Vec<(ID, ID)> = jobs.iter_mut()
                                .filter(|job| job.task_id() < k as ID)
                                .map(|job| (job.task_id(), job.id()))
                                .collect();

        // return the smallest task_id if it is below k
        if smallers.len() > 0 {
            let (_, job_id) = smallers.iter().min_by_key(|&&(task_id, _)| task_id).unwrap();
            return Some(*job_id);
        }

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

    /// Test for possible basic shortcuts, if there is not -> None
    pub fn test_shortcuts(&self) -> Option<SchedulingCode> {
        if self.task_set.is_empty() {
            return Some(SchedulingCode::SchedulableShortcut);
        }

        
        if !self.task_set.is_feasible() {
            return Some(SchedulingCode::UnschedulableShortcut);
        }

        
        if self.task_set.checking_schedulability() { // TODO check ?
            if self.task_set.schedulability_proven(&self.task_set) {
                return Some(SchedulingCode::SchedulableShortcut);
            } else {
                return Some(SchedulingCode::UnschedulableShortcut);
            }
        }
        
        return None;
    }

    pub fn simulate_partitionned(&mut self) -> SchedulingCode {
        let result = self.simulate_edfk(1);

        // default response for edfk
        if result != SchedulingCode::CannotTell {
            return result;
        }
        
        SchedulingCode::SchedulableSimulated
    }


    pub fn simulate_edfk(&mut self, k: usize) -> SchedulingCode {

        if let Some(result_shortcut) = self.test_shortcuts() {
            // println!("result != None : {:?}", result_shortcut);
            return result_shortcut;
        }

        let mut queue: Vec<Job> = Vec::new();
        let feasibility_interval = self.task_set.feasibility_interval(&self.task_set).1;
        
        while self.current_time < feasibility_interval {
            // Try to release new jobs at time `t`
            queue.extend(self.task_set.release_jobs(self.current_time));
            
            // Check for missed deadlines
            if queue.iter().any(|job| job.deadline_missed(self.current_time)) {
                println!("time {:?}", self.current_time);
                return SchedulingCode::UnschedulableSimulated;
            }

            // Clone the job to be scheduled to avoid multiple mutable borrows
            if let Some(index_elected) = self.schedule(&mut queue, k){
                if let Some(elected) = queue.get_mut(index_elected as usize) {
                    elected.schedule(1);
                }
            }
        

            // Filter out completed jobs
            queue = queue.into_iter().filter(|job| !job.is_complete()).collect();

            self.current_time = self.current_time + 1
        }
        
        SchedulingCode::CannotTell

    }
}

    