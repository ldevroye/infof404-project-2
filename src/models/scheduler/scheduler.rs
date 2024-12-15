use crate::{Job, SchedulingCode, TaskSet, TimeStep, ID};
use crate::constants::EDFVersion;
use crate::scheduler::core::Core;

use std::cmp::Ordering;
use std::process::exit;


pub struct Scheduler{
    task_set: TaskSet,
    version: EDFVersion,
    num_cores: usize,
    num_threads: usize,
    cores: Vec<Core>,
    heuristic: String,
    sorting_order: String, 
}


impl Scheduler {
    pub fn new(task_set: TaskSet, num_cores: usize, num_threads:usize, heuristic: String, sorting_order: String) -> Self {
        Self {
            task_set,
            version : EDFVersion::Partitioned,
            num_cores,
            num_threads,
            cores: (1..=num_cores).map(|id| Core::new(id as ID)).collect(),
            heuristic,
            sorting_order,
        }

        
    }
    pub fn set_version(&mut self, new_version: EDFVersion) {
        self.version = new_version;
    }


    pub fn check_global_edf_schedulability(&self) -> bool {
        let num_cores_f64 = self.num_cores as f64;
        self.task_set.utilisation() <= num_cores_f64 - (num_cores_f64 - 1.0) * self.task_set.max_utilisation()

    }

    pub fn check_edf_k_schedulability(&self) -> bool {
        let k = match self.version {
            EDFVersion::EDFk(value) => {value} 
            _ => {1} // edfk(1) = edf
        };

        if k >= self.task_set.len() - 1 {
            return false;
        }
        
        if self.task_set.get_task(k).unwrap().utilisation() == 1.0 {
            return false;
        }
        
        return self.num_cores as f64 >= k as f64 +
                            self.task_set.get_task_utilisation(k + 1).unwrap() / 
                            (1.0 - self.task_set.get_task_utilisation(k ).unwrap());
    }


    /// Partition the `n` tasks over `m` processors according the the `heuristic` to follow and the sorting `order`
    ///
    /// # Arguments
    ///
    /// * `task` - the set of tasks to partition of size n.
    /// * `m` - the number of available processors.
    /// * `heuristic` - the heuristic to fill the processors.
    /// * `order` - increasing of decreasing order of utilisation.
    ///
    /// # Returns 
    ///
    /// A partition such that the ith vector of tasks is to be done by the ith processor
    /// 
    /// Example for 3 tasks and 2 processors : [[Task 3, Task 1], [Task 2]]
    fn partition_tasks(&mut self) -> Vec<TaskSet> {

        if self.sorting_order == "du" {
            self.task_set.get_tasks_mut().sort_by(|a, b| b.utilisation().partial_cmp(&a.utilisation()).unwrap_or(Ordering::Equal));
        } else if self.sorting_order == "iu" {
            self.task_set.get_tasks_mut().sort_by(|a, b| a.utilisation().partial_cmp(&b.utilisation()).unwrap_or(Ordering::Equal));
        } else {
            panic!("Unknown sorting order")
        }
    
        //let mut partitions: Partition = Partition::new(m); // partition of each task per worker
        let mut partitions: Vec<TaskSet> = vec![TaskSet::new_empty(); self.num_cores];
        let num_task = self.task_set.get_tasks_mut().len();
        match self.heuristic.as_str() {
            "ff" => {
                for task in self.task_set.get_tasks_mut().iter() {
                    for task_set in partitions.iter_mut() {
                        if task_set
                        .iter()
                        .map(|t| t.utilisation())
                        .sum::<f64>() + task.utilisation()
                         <= 1.0 {
                            task_set.add_task(task.clone());
                            break;
                        }
                    }
                }
            }
            "nd" => {
                let mut current_partition = 0;
                for task in self.task_set.get_tasks_mut().iter() {
                    if partitions[current_partition]
                        .iter()
                        .map(|t| t.utilisation())
                        .sum::<f64>()
                        + task.utilisation()
                        > 1.0
                    {
                        current_partition = (current_partition + 1) % self.num_cores;
                    }
                    partitions[current_partition].add_task(task.clone());
                }
            }
            "bf" => {
                for task in self.task_set.get_tasks_mut().iter() {
                    let mut best_partition: Option<usize> = None;
                    let mut min_slack = f64::MAX;
    
                    for (i, task_set) in partitions.iter().enumerate() {
                        let slack = 1.0 - task_set.iter().map(|t| t.utilisation()).sum::<f64>();
                        if slack >= task.utilisation() && slack < min_slack {
                            best_partition = Some(i);
                            min_slack = slack;
                        }
                    }
    
                    if let Some(best) = best_partition {
                        partitions[best].add_task(task.clone());
                    }
                }
            }
            "wf" => {
                for task in self.task_set.get_tasks_mut().iter() {
                    let mut worst_partition: Option<usize> = None;
                    let mut max_slack = f64::MIN;
    
                    for (i, task_set) in partitions.iter().enumerate() {
                        let slack = 1.0 - task_set.iter().map(|t| t.utilisation()).sum::<f64>();
                        if slack >= task.utilisation() && slack > max_slack {
                            worst_partition = Some(i);
                            max_slack = slack;
                        }
                    }
    
                    if let Some(worst) = worst_partition {
                        partitions[worst].add_task(task.clone());
                    }
                }
            }
            _ => panic!("Unknown heuristic"),
        }
    
        let num_task_computed:usize = partitions.iter().map(|taskset| taskset.len()).sum();
    
        if num_task > num_task_computed {
            eprintln!("Too much task ({:?}, {:?} attached to a processor) for the {:?} processors", num_task, num_task_computed, self.num_cores);
            println!("{:#?}", partitions);
            exit(SchedulingCode::UnschedulableShortcut as i32)
        }
        partitions
    }


    /// Main function for the partitionned EDF
    pub fn compute_partitionned(&mut self) -> SchedulingCode {
        let mut processor_done = 0 as usize;
        let mut thread_running = 0 as usize;
    
        // Allow parameters to another fn()
        //let partition_mutex = Arc::new(Mutex::new(partition));
    
        // Use Arc to share ownership of partition across threads
        //let threads_mutex: Arc<Mutex<Vec<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(vec![]));

        let partition = self.partition_tasks();
        // println!("Partition : {:#?}", partition);

        // clone so that the scheduler and the the cores have different tasksets
        self.cores = (1..=self.num_cores)
        .map(|id| Core::new_task_set(id as ID, partition[id-1].clone()))
        .collect(); 
    
        println!("Cores : {:#?}", self.cores);

        let mut result = SchedulingCode::SchedulableShortcut;

        while processor_done < self.num_cores {
            let current_core = self.cores.get_mut(processor_done).unwrap();
            let resp = current_core.simulate_partitionned();            

            // if any is not schedulable then -> not schedulable
            if ! (resp == SchedulingCode::SchedulableShortcut || resp == SchedulingCode::SchedulableSimulated) { 
                println!("Taskset not schedulable");
                return resp;
            }
            
            // if any has to be simulated then -> ok simulated
            if resp == SchedulingCode::SchedulableSimulated {
                result = resp
            } 
            
            processor_done += 1;
        }

        return result;

            /* 
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
        */
    }

    /// edf_k job scheduling : gives at most one job_id per processor
    pub fn schedule_jobs(queue: &Vec<Job>, k: usize) -> Option<ID> {

        // normal edf scheduling

        if queue.is_empty() {
            return None; 
        }

        // Initialize with the absolute deadline of the first job in the list.
        
        if queue.len() == 0 {
            return None;
        }
        let mut smallest = queue[0].absolute_deadline();
        let mut index_to_ret: ID = 0;

        // Iterate over the jobs to find the one with the earliest absolute deadline.
        let jobs_size = queue.len();
        for i in 0..jobs_size {
            if smallest == TimeStep::MIN {break;}

            let current_deadline = queue[i].absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = i as ID;
            }
        }

        Some(index_to_ret)

    }
    
    pub fn compute_edfk(&mut self) -> SchedulingCode {

        let k = match self.version {
            EDFVersion::EDFk(value) => {value}
            _ => {1}
        };

        let mut result = SchedulingCode::SchedulableShortcut;
        if ! self.check_edf_k_schedulability() {
            return SchedulingCode::UnschedulableShortcut;
        }
        
        let mut time: TimeStep = 0;
        let max_time: TimeStep = self.task_set.feasibility_interval().1;

        while time < max_time {
            let mut queue: Vec<Job> = Vec::new();
            let mut job_to_add = self.task_set.release_jobs(time);

            // set the jobs with 
            for job in job_to_add.iter_mut() {
                if job.task_id() < k as ID {
                    job.set_deadline_inf();
                }
            }

            queue.extend(job_to_add);
            

            for index_core in 0..self.num_cores-1 {
                if queue.is_empty() {break;}

                let current_core = self.cores.get_mut(index_core).unwrap();
                let elected_index: ID = Scheduler::schedule_jobs(&queue, k).unwrap();

                let current_job = queue.get(elected_index).unwrap();

                // check if assigned 
                // TODO check if the job has lower priority then replace
                // 
                if !current_core.is_assigned() {
                    current_core.add_job(current_job.clone(), self.task_set.get_task_by_id(current_job.task_id()).unwrap().clone())
                }

                queue.remove(elected_index);

            }

            time += 1
        }

        return SchedulingCode::CannotTell;

        // looks like Core.simulate()
        /* 
        if let Some(result_shortcut) = self.test_shortcuts() {
            // println!("result != None : {:?}", result_shortcut);
            return result_shortcut;
        }

        let mut queue: Vec<Job> = Vec::new();
        let feasibility_interval = self.task_set.feasibility_interval(&self.task_set).1;
        
        while self.current_time < feasibility_interval {

            let result = self.simulate_step(1);
            if result != None {
                return result.unwrap();
            }    
        }
        
        */
        
        return result;
    }

    pub fn compute_global(&mut self) -> SchedulingCode {
        let mut result = SchedulingCode::SchedulableShortcut;

        if !self.check_global_edf_schedulability() {
            return SchedulingCode::UnschedulableShortcut;
        }

        return SchedulingCode::CannotTell;
    }

    /// Hub function to chose which version to use
    pub fn test_task_set(&mut self) -> SchedulingCode {

        match self.version {
            EDFVersion::EDFk(value) => {
                return self.compute_edfk();
            }
            EDFVersion::Global => {
                todo!()
            }
            EDFVersion::Partitioned => {
                return self.compute_partitionned();
            }

            _ => {
                eprintln!("Unknow EDF version");
                SchedulingCode::CannotTell
            }
        }

        
    }
}