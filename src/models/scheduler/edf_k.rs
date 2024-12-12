use super::scheduler::Scheduler;
use crate::{Job, TaskSet, TimeStep, ID};

pub struct EDFK {
    num_workers: usize,
}

impl Scheduler for EDFK {
    fn new(num_workers: usize) -> Self {
        Self { num_workers }
    }

    fn schedule<'a>(&'a self, jobs: &'a mut Vec<Job>) -> Option<ID> {
        todo!()
    }

    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep) {
        //[𝑂max, 𝑂max + 2𝑃)
        todo!()
    }
}