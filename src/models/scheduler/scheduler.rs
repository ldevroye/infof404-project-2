use std::ops::Index;

use crate::{TaskSet, ID, TimeStep, Job, SchedulingCode};


pub trait Scheduler {
    fn schedule<'a>(&'a self, jobs: &'a mut Vec<Job>) -> Option<ID>;
    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep);
    fn checking_schedulability(&self) -> bool {
        false
    }
    fn schedulability_proven(&self, _: &TaskSet) -> bool {
        false
    }

    fn check_global_edf_schedulability(&self, taskset: &TaskSet, num_cores: usize) -> bool {
        let num_cores_f64 = num_cores as f64;
        taskset.utilisation() <= num_cores_f64 - (num_cores_f64 - 1.0) * taskset.max_utilisation()
    }

    fn check_edf_k_schedulability(&self, k: usize, taskset: &mut TaskSet, num_cores: usize) -> bool {
        if k >= taskset.len() - 1 {
            std::process::exit(SchedulingCode::UnschedulableShortcut as i32);
        }
        
        if taskset.get_task(k).unwrap().utilisation() == 1.0 {
            return false;
        }
        
        return num_cores as f64 >= k as f64 +
                            taskset.get_task(k + 1).unwrap().utilisation() / 
                            (1.0 - taskset.get_task(k).unwrap().utilisation());
    }
}