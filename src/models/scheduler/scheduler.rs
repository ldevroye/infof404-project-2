use crate::Job;
use crate::TaskSet;
use crate::TimeStep;

pub trait Scheduler {
    fn schedule<'a>(&'a mut self, jobs: &'a mut Vec<Job>) -> Option<&'a mut Job>;
    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep);
    fn checking_schedulability(&self) -> bool {
        false
    }
    fn schedulability_proven(&self, _: &TaskSet) -> bool {
        false
    }
}