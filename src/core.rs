use crate::{scheduler, SchedulingCode, TaskSet};
use crate::scheduler::{EarliestDeadlineFirst, Scheduler};

pub fn simulation(mut taskset: TaskSet) -> SchedulingCode {
    let scheduler = EarliestDeadlineFirst;

    if taskset.is_empty() || !taskset.is_feasible() {
        return SchedulingCode::SchedulableShortcut;
    }
    
    if scheduler.checking_schedulability() {
        if scheduler.schedulability_proven(&taskset) {
            return SchedulingCode::SchedulableShortcut;
        } else {
            return SchedulingCode::UnschedulableShortcut;
        }
    }

    let mut queue = Vec::new();
    let feasibility_interval = scheduler.feasibility_interval(&taskset).1;
    
    for t in 0..feasibility_interval {
        // Release new jobs at time `t`
        queue.extend(taskset.release_jobs(t));
        
        // Check for missed deadlines
        if queue.iter().any(|job| job.deadline_missed(t)) {
            return SchedulingCode::UnschedulableSimulated;
        }

        // Clone the job to be scheduled to avoid multiple mutable borrows
        if let Some(elected) = scheduler.schedule(&mut queue) {
            elected.schedule(1);
        }

        // Filter out completed jobs
        queue = queue.into_iter().filter(|job| !job.is_complete()).collect();
    }

    SchedulingCode::SchedulableSimulated
}