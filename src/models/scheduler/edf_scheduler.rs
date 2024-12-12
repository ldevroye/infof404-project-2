use clap::Error;

use super::scheduler::Scheduler;
use crate::{models::job, Job, TaskSet, TimeStep, ID};

pub struct EarliestDeadlineFirst;

impl Scheduler for EarliestDeadlineFirst {
    fn schedule<'a>(&'a self, jobs: &'a mut Vec<Job>) -> Option<TimeStep> {
        if jobs.is_empty() {
            return None;
        }

        // Initialize with the absolute deadline of the first job in the list.
        let mut smallest = jobs[0].absolute_deadline();
        let mut index_to_ret = 0;

        // Iterate over the jobs to find the one with the earliest absolute deadline.
        let jobs_size = jobs.len();
        for i in 0..jobs_size {
            let current_deadline = jobs[i].absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = i;
            }
        }

        Some(index_to_ret)
        
    }
    
    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep) {
        let w_0 = taskset
            .iter()
            .map(|task| task.wcet())
            .sum::<TimeStep>();

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