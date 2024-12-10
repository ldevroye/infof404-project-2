use super::scheduler::Scheduler;
use crate::{Job, Task, TaskSet, TimeStep};

pub struct DeadlineMonotonic;

impl DeadlineMonotonic {
    fn response_time(&self, task: &Task, taskset: &TaskSet) -> TimeStep {
        // TODO
        0
    }
}

impl Scheduler for DeadlineMonotonic {
    fn schedule<'a>(&'a mut self, jobs: &'a mut Vec<Job>) -> Option<&'a mut Job> {
        if jobs.is_empty() {
            return None;
        }

        jobs.sort_by(|a, b| a.task().deadline().cmp(&b.task().deadline()));
        jobs.first_mut()
    }

    fn feasibility_interval(&self, taskset: &TaskSet) -> (TimeStep, TimeStep) {
        let max_deadline = taskset.iter().map(|t| t.deadline()).max().unwrap_or(0);
        (0, max_deadline)
    }

    fn checking_schedulability(&self) -> bool {
        true
    }

    fn schedulability_proven(&self, taskset: &TaskSet) -> bool {
        taskset.iter().all(|task| {
            let response_time = self.response_time(task, taskset);
            response_time <= task.deadline()
        })
    }
}