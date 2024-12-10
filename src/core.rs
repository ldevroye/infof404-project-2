use crate::{scheduler, SchedulingCode, TaskSet};
use crate::scheduler::{EarliestDeadlineFirst, Scheduler};

pub fn simulation(mut taskset: TaskSet, num_processor: u32) -> SchedulingCode {
    let scheduler = EarliestDeadlineFirst;

    if taskset.is_empty() {
        println!("empty");
        return SchedulingCode::SchedulableShortcut;
    }

    if !taskset.is_feasible(num_processor) {
        println!("stop feasible");
        return SchedulingCode::UnschedulableShortcut
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