use super::scheduler::Scheduler;
use crate::{Job, TimeStep, TaskSet, ID};

/**
 * EDF with task migrations of the m jobs on the m cores.
 * 
 * We neglect migration time
 */
pub struct EDFGlobal {
    num_workers: usize,
}

impl Scheduler for EDFGlobal {
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