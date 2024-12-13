use crate::{TaskSet, ID, TimeStep, Job};


pub trait Scheduler {
    fn schedule<'a>(&'a self, jobs: &'a mut Vec<Job>) -> Option<ID>;
    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep);
    fn checking_schedulability(&self) -> bool {
        true
    }
    fn schedulability_proven(&self, _: &TaskSet) -> bool {
        false
    }
}