use super::scheduler::Scheduler;
use crate::{Job, TimeStep};

pub struct EarliestDeadlineFirst;

impl Scheduler for EarliestDeadlineFirst {
    fn schedule<'a>(&'a mut self, jobs: &'a mut Vec<Job>) -> Option<&'a mut Job> {
        if jobs.is_empty() {
            return None;
        }

        jobs.sort_by(|a, b| a.deadline().cmp(&b.deadline()));
        jobs.first_mut()
    }
    fn feasibility_interval(&self, taskset: &crate::TaskSet) -> (TimeStep, TimeStep) {
        let w_0 = taskset
            .iter()
            .map(|task| task.wcet())
            .sum::<u32>();

        let mut w_k = w_0;

        loop {
            let w_k_next = taskset
                .iter()
                .map(|task| {
                    let ceiling_term = (w_k + task.period() - 1) / task.period();
                    ceiling_term * task.wcet()
                })
                .sum();

            if w_k_next == w_k {
                break (w_0, w_k);
            } else {
                w_k = w_k_next;
            }
        }
    }
}