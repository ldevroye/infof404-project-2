use super::scheduler::Scheduler;
use crate::{Job, TimeStep, TaskSet};

/**
 * EDF with task migrations of the m jobs on the m cores.
 * 
 * We neglect migration time
 */
pub struct EDFGlobal {
    cores: u32,
}

impl Scheduler for EDFGlobal {
    fn schedule<'a>(&'a self, jobs: &'a mut Vec<Job>) -> Option<TimeStep> {
        todo!()
    }

    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep) {
        todo!()
    }
}