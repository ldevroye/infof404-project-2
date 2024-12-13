use crate::{scheduler, taskset, Job, Partition, SchedulingCode, Task, TaskSet, ID};
use crate::scheduler::{EarliestDeadlineFirst, Scheduler};
use std::thread;
use std::sync::{Arc, Mutex};

pub fn simulation(partition: &mut Partition, thread_number: usize) -> SchedulingCode {
    let processor_number = partition.processor_number();

    let mut processor_done = 0 as usize;
    let mut thread_running = 0 as usize;

    // Specify the type for Mutex to avoid inference issues
    let threads: Arc<Mutex<Vec<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(vec![]));

    while processor_done < processor_number {
        {
            let mut threads_lock = threads.lock().unwrap();

            // Check if any thread has completed
            threads_lock.retain(|handle: _| {
                if handle.is_finished() {
                    thread_running -= 1;
                    false // Remove finished thread from the list
                } else {
                    true // Keep running threads
                }
            });
        }

        if thread_running < thread_number {
            // Spawn a new thread
            let threads_clone = Arc::clone(&threads);
            let handle = thread::spawn(move || {
                println!("Processor {} is running.", processor_done + 1);
                let taskset = partition.get(processor_done).unwrap();
                simulate(taskset); // Simulate processing
                println!("Processor {} is done.", processor_done + 1);
            });

            {
                let mut threads_lock = threads_clone.lock().unwrap();
                threads_lock.push(handle);
            }

            thread_running += 1;
            processor_done += 1;
        } else {
            // If max threads are running, wait a bit
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    // Wait for all threads to complete
    for handle in Arc::try_unwrap(threads).unwrap().into_inner().unwrap() {
        handle.join().unwrap();
    }

    SchedulingCode::SchedulableSimulated
}

fn simulate(taskset: &mut TaskSet) -> SchedulingCode {
    let scheduler = EarliestDeadlineFirst;

    if taskset.is_empty() {
        return SchedulingCode::SchedulableShortcut;
    }

    if !taskset.is_feasible(1) {
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