use crate::{scheduler, Job, SchedulingCode, TaskSet, Task, ID};
use crate::scheduler::{EarliestDeadlineFirst, Scheduler};

pub fn simulation(mut taskset: TaskSet, num_workers: ID) -> SchedulingCode {
    let scheduler = EarliestDeadlineFirst;


    if taskset.is_empty() {
        return SchedulingCode::SchedulableShortcut;
    }

    if !taskset.is_feasible(num_workers) {
        return SchedulingCode::UnschedulableShortcut
    }
    
    if scheduler.checking_schedulability() { // TODO check ?
        if scheduler.schedulability_proven(&taskset) {
            return SchedulingCode::SchedulableShortcut;
        } else {
            return SchedulingCode::UnschedulableShortcut;
        }
    }

    let mut queue: Vec<Job> = Vec::new();
    let feasibility_interval = scheduler.feasibility_interval(&taskset).1;
    
    for t in 0..feasibility_interval {
        // Try to release new jobs at time `t`
        queue.extend(taskset.release_jobs(t));
        
        // Check for missed deadlines
        if queue.iter().any(|job| job.deadline_missed(t)) {
            println!("time {:?}", t);
            return SchedulingCode::UnschedulableSimulated;
        }

        // Clone the job to be scheduled to avoid multiple mutable borrows
        if let Some(index_elected) = scheduler.schedule(&mut queue){
            if let Some(elected) = queue.get_mut(index_elected as usize) {
                elected.schedule(1);
            }
        }
    

        // Filter out completed jobs
        queue = queue.into_iter().filter(|job| !job.is_complete()).collect();
    }
    SchedulingCode::SchedulableSimulated
}