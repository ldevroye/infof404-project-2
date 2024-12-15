use crate::models::job;
use crate::{partition, Job, SchedulingCode, TaskSet, TimeStep, ID};
use crate::constants::{CoreValue, Version, Heuristic};
use crate::scheduler::{Core};//, GlobalCore, PartitionnedCore} 

use std::cmp::Ordering;
use std::process::exit;
use std::result;
use std::thread::current;


pub struct Scheduler{
    task_set: TaskSet,
    version: Version,
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
            version : Version::Partitioned,
            num_cores,
            num_threads,
            cores: (1..=num_cores).map(|id| Core::new(id as ID)).collect(),
            heuristic,
            sorting_order,
        }

        
    }
    pub fn set_version(&mut self, new_version: Version) {
        self.version = new_version;
    }

    /// Returns the k of EDF(k) (or 1 if the version is not edf(k))
    pub fn get_k(&self) -> usize {
        match self.version {
            Version::EDFk(value) => {value} 
            _ => {1} // edfk(1) = edf
        }
    }


    /// Test if the taskset can be schedulable on global edf  
    pub fn check_global_edf_schedulability(&self) -> bool {
        let num_cores_f64 = self.num_cores as f64;
        self.task_set.utilisation() <= num_cores_f64 - (num_cores_f64 - 1.0) * self.task_set.max_utilisation()

    }

    /// Test if the taskset can be schedulable on edf k
    pub fn check_edf_k_schedulability(&self) -> bool {
        
        let k = self.get_k();

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


    /// edf_k job scheduling : gives the job with lowest shown deadline
    pub fn schedule_jobs(queue: &Vec<(Job, usize)>, k: usize) -> Option<ID> {

        // Initialize with the absolute deadline of the first job in the list.
        
        if queue.len() == 0 {
            return None;
        }

        let mut smallest = TimeStep::MAX;
        let mut index_to_ret: Option<ID> = None;

        // Iterate over the jobs to find the one with the earliest absolute deadline.
        let jobs_size = queue.len();
        for i in 0..jobs_size {
            if smallest == TimeStep::MIN {break;}

            let (job, id_core) = &queue[i];

            if *id_core != 0 { // job already taken by a core
                continue;
            }

            let current_deadline = job.absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = Some(i as ID);
            }
        }

        index_to_ret

    }
    
    /// compute the task set among all the cores using edf(k), with possible migrations inbetween cores during the simulation
    pub fn compute_edfk(&mut self) -> SchedulingCode {

        if ! self.check_edf_k_schedulability() {
            return SchedulingCode::UnschedulableShortcut;
        }

        let k = self.get_k();
        let mut queue: Vec<(Job, ID)> = Vec::new();
        let mut result = SchedulingCode::SchedulableSimulated;
        
        let (mut time, max_time) = self.task_set.feasibility_interval_global();
        println!("interval : [{:?}, {:#?}]", time, max_time);
        let mut threash_hold = max_time/100;

        self.cores.iter_mut().for_each(|core| core.set_time(time));

        while time < max_time {
            let mut job_to_add = self.task_set.release_jobs(time);

            // set the jobs deadline with task_id < k to -inf 
            for job in job_to_add.iter_mut() {
                if job.task_id() < k as ID {
                    job.set_deadline_inf();
                }
            }

            // cores have id >= 1 so 0 = not handled
            queue.extend(job_to_add.into_iter().map(|job| (job, 0))); 
            
            // try to add 1 job with lowest prio to each core
            for index_core in 0..self.num_cores-1 {
                let any_available_core = self.cores.iter().any(|core| !core.is_assigned());
                let current_core = self.cores.get_mut(index_core).unwrap();

                // if queue empty or the job in the core is already == -inf
                if queue.is_empty() {
                    break;
                } else if (current_core.current_job_is_inf().is_some()) {
                    if current_core.current_job_is_inf().unwrap() {
                        continue;
                    }
                }

                let result_schedule = Scheduler::schedule_jobs(&queue,k);
                if result_schedule.is_none() { // there is no job not handleled by a core
                    break;
                }

                let elected_index: ID = result_schedule.unwrap();

                // Extract the `current_job` and `core_id` without holding a mutable borrow
                let (current_job, core_id) = {
                    let (job, core) = queue.get_mut(elected_index).unwrap();
                    (job.clone(), *core)
                };

                // Check if the core is not assigned 
                // or if the new job's deadline is earlier and there is no other core available
                if !current_core.is_assigned() || 
                (!any_available_core && current_core.is_assigned() && 
                    current_core.current_job_deadline().unwrap() > current_job.absolute_deadline() &&
                    current_core.current_job_deadline().is_some()) {

                    if current_core.is_assigned() {
                        current_core.remove(current_job.task_id());

                        // Perform the second mutable operation after the first borrow ends
                        if let Some((_, core_ref)) = queue
                                                                .iter_mut()
                                                                .find(|(_, core_ref)| *core_ref == current_core.id()) {
                            //print!("putting {} to 0", core_ref);
                            *core_ref = 0; // Reset the core ID
                        }
                    }

                    // Reborrow `queue` mutably to update the core_id for the elected_index
                    {
                        let (_, core_id_mut) = queue.get_mut(elected_index).unwrap();
                        //print!("(and maybe) put {} to {}\n", core_id_mut, current_core.id());
                        *core_id_mut = current_core.id(); // Assign the current core ID
                    }

                    let task_clone = self.task_set.get_task_by_id(current_job.task_id()).unwrap().clone();
                    // current_job is already cloned
                    current_core.add_job(current_job, task_clone);
                }


            }

            // simulate 1 step in each cores
            for index_core in 0..self.num_cores-1 {

                let current_core = self.cores.get_mut(index_core).unwrap();
                let resp = current_core.simulate_step(k);
                if resp == CoreValue::Missed {
                    result = SchedulingCode::UnschedulableSimulated;
                    break;
                    
                } else if resp == CoreValue::Commplete {
                    // remove the job complete from queue
                    queue.retain(|(_, id)| *id != current_core.id());
                } else { // running, you have to simu
                    queue.iter_mut()
                    .for_each(|(job, core_id)| if *core_id == current_core.id() 
                    {job.set_remaining_time(current_core.get_remaining_time());});
                }
            }

            // test_deadlines for each jobs not handled
            for (job, id) in queue.iter() {
                if *id == 0 { // unhandled
                    if job.deadline_missed(time) {
                        result = SchedulingCode::UnschedulableSimulated;
                        println!("Unhandled deadline missed ({}) at time {}, task_id {}", job.absolute_deadline(), time, job.task_id());
                        break;
                    }
                }
            }

            if result == SchedulingCode::UnschedulableSimulated {
                break;
            } 

            time += 1;
            if (time > threash_hold) {
                println!("{}% done", (time*100)/max_time);
                threash_hold += max_time/100;
            }

        }

        return result;

    }



    pub fn compute_global(&mut self) -> SchedulingCode {
        if !self.check_global_edf_schedulability() {
            return SchedulingCode::UnschedulableShortcut;
        }

        self.compute_edfk() // k will be 1
    }


    /// DM (Deadline Monotonic) scheduling inspired by compute_edfk.
    pub fn compute_dm(&mut self) -> SchedulingCode {

        self.task_set.get_tasks_mut().sort_by(|a, b| a.deadline().cmp(&b.deadline()));

        let mut result = SchedulingCode::SchedulableSimulated;
        
        let (mut time, max_time) = self.task_set.feasibility_interval_global();
        let mut thresh_hold = max_time/100;
        println!("time : [{},{}]", time, max_time);

        while time < max_time {
            // Release jobs that arrive at 'time'
            let mut job_to_add = self.task_set.release_jobs(time);

            // For DM, priorities are fixed based on deadline order determined above.
            let mut queue: Vec<(Job, ID)> = job_to_add.into_iter().map(|job| (job, 0)).collect();

            // Assign jobs to cores
            for index_core in 0..self.num_cores-1 {
                let any_available_core = self.cores.iter().any(|core| !core.is_assigned());
                let current_core = self.cores.get_mut(index_core).unwrap();

                // if queue empty or the job in the core is already == -inf
                if queue.is_empty() {
                    break;
                } else if (current_core.current_job_is_inf().is_some()) {
                    if current_core.current_job_is_inf().unwrap() {
                        continue;
                    }
                }

                let result_schedule = Scheduler::schedule_jobs_dm(&queue);
                if result_schedule.is_none() { // there is no job not handleled by a core
                    break;
                }

                let elected_index: ID = result_schedule.unwrap();

                // Extract the `current_job` and `core_id` without holding a mutable borrow
                let (current_job, core_id) = {
                    let (job, core) = queue.get_mut(elected_index).unwrap();
                    (job.clone(), *core)
                };

            
                let (current_job, core_id) = &mut queue[elected_index];

                // Check if we should replace current job
                if !current_core.is_assigned() ||
                    (!any_available_core &&
                    current_core.is_assigned() &&
                    current_core.current_job_task_deadline().unwrap() > current_job.task_deadline() &&
                    current_core.current_job_deadline().is_some())
                {
                    if current_core.is_assigned() {
                        current_core.remove(current_job.task_id());
                    }

                    *core_id = current_core.id();
                    let task_clone = self.task_set.get_task_by_id(current_job.task_id()).unwrap().clone();
                    current_core.add_job(current_job.clone(), task_clone);
                }
                
            }
        
            // Simulate one time unit
            for index_core in 0..self.num_cores-1 {
                let current_core = self.cores.get_mut(index_core).unwrap();
                let resp = current_core.simulate_step(1);

                if resp == CoreValue::Missed {
                    result = SchedulingCode::UnschedulableSimulated;
                } else if resp == CoreValue::Commplete {
                    // Remove completed jobs from queue
                    queue.retain(|(_, id)| *id != current_core.id());
                }
            }

            // Check deadlines for unhandled jobs
            for (job, id) in &queue {
                if *id == 0 && job.deadline_missed(time) {
                    result = SchedulingCode::UnschedulableSimulated;
                    break;
                }
            }

            if result == SchedulingCode::UnschedulableSimulated {
                break;
            }

            time += 1;
            if (time > thresh_hold) {
                println!("{}% done", (time*100)/max_time);
                thresh_hold += max_time/100;
            }
        }

        result
    }




    /// Helper function for DM to pick the job with the highest priority (lowest relative deadline task).
    fn schedule_jobs_dm(queue: &Vec<(Job, ID)>) -> Option<ID> {
        if queue.is_empty() {
            return None;
        }

        let mut best_priority = TimeStep::MAX;
        let mut index_to_ret: Option<ID> = None;

        for (i, (job, core_id)) in queue.iter().enumerate() {
            if *core_id != 0 {
                // Already taken by a core
                continue;
            }

            // Get task priority (ie, task deadline)
            let task_priority = queue.iter().map(|(job, _)| job.task_deadline()).min().unwrap();
            if task_priority < best_priority {
                best_priority = task_priority;
                index_to_ret = Some(i as ID);
            }
        }

        index_to_ret
    }


    /// Hub function to chose which version to use
    pub fn test_task_set(&mut self) -> SchedulingCode {
        let result: SchedulingCode;
        match self.version {
            Version::EDFk(value) => {
                result = self.compute_edfk();
            }
            Version::Global => {
                result = self.compute_global();
            }
            Version::Partitioned => {
                result = self.compute_partitionned();
            }
            Version::GlobalDM => {
                result = self.compute_dm();
            }

            _ => {
                eprintln!("Unknow EDF version");
                return SchedulingCode::CannotTell;
            }
        }

        let migrations: usize = self.cores.iter().map(|core| core.get_migrations()).sum();
        print!("Number migrations : {}", migrations);
        return result;
        
    }
}