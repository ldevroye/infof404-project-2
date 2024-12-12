use clap::Error;

use crate::{Task, Job, TimeStep, TaskSet, constants, ID};

pub trait Scheduler {
    fn new(num_workers: usize) -> Self;
    fn schedule<'a>(&'a self, jobs: &'a mut Vec<Job>) -> Option<ID>;
    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep);
    fn checking_schedulability(&self) -> bool {
        false
    }
    fn schedulability_proven(&self, _: &TaskSet) -> bool {
        false
    }
}