use crate::{Job, Partition, SchedulingCode, TaskSet};
use crate::scheduler::{EarliestDeadlineFirst, Scheduler};
use std::thread;
use std::sync::{Arc, Mutex};

pub fn simulation(partition: Partition, thread_number: usize) -> SchedulingCode {
    let processor_number = partition.processor_number();

    let mut processor_done = 0 as usize;
    let mut thread_running = 0 as usize;

    // Allow parameters to another fn()
    let partition = Arc::new(Mutex::new(partition));

    // Use Arc to share ownership of partition across threads
    let threads: Arc<Mutex<Vec<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(vec![]));

    while processor_done < processor_number {
        {
            let mut threads_lock = threads.lock().unwrap();

            // Check if any thread has completed
            threads_lock.retain(|handle| {
                if handle.is_finished() {
                    thread_running -= 1;
                    false // Remove finished thread from the list
                } else {
                    true // Keep running threads
                }
            });
        }

        if thread_running < thread_number {
            // Clone the Arc for each new thread
            let partition_clone = Arc::clone(&partition);

            let handle = thread::spawn(move || {
                // Lock the partition inside the thread to safely access it
                let mut partition_lock = partition_clone.lock().unwrap();
                let taskset = partition_lock.get_mut(processor_done).unwrap();
                println!("Processor {} is running.", processor_done + 1);
                simulate(taskset); // Simulate processing of each processor
                println!("Processor {} is done.", processor_done + 1);
            });

            {
                let mut threads_lock = threads.lock().unwrap();
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

fn simulate<'a>(taskset: &'a mut TaskSet) -> SchedulingCode {
    let scheduler = EarliestDeadlineFirst;
    println!("ok?");
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
                println!("wow");
                elected.schedule(1);
            }
        }
    

        // Filter out completed jobs
        queue = queue.into_iter().filter(|job| !job.is_complete()).collect();
    }
    SchedulingCode::SchedulableSimulated
}