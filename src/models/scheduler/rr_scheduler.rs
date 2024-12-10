use super::scheduler::Scheduler;
use crate::{Job, lcm};

pub struct RoundRobin {
    last_index: usize,
}

impl RoundRobin {
    pub fn new() -> Self {
        Self { last_index: 0 }
    }
}

impl Scheduler for RoundRobin {
    fn schedule<'a>(&'a mut self, jobs: &'a mut Vec<Job>) -> Option<&'a mut Job> {
        if jobs.is_empty() {
            return None;
        }
        // Select the next job in a circular fashion
        self.last_index = (self.last_index + 1) % jobs.len();
        jobs.get_mut(self.last_index)
    }
    fn feasibility_interval(&self, taskset: &crate::TaskSet) -> (crate::TimeStep, crate::TimeStep) {
        let lcm_value = lcm::multiple_lcm(&taskset.iter().map(|task| task.period()).collect::<Vec<_>>());
        (0, lcm_value)
    }
}