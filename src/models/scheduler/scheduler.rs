use crate::{Job, SchedulingCode, TaskSet, TimeStep, ID};
use crate::constants::EDFVersion;
use crate::scheduler::Core; // Importing Core module

use std::cmp::Ordering;
use std::process::exit;

#[allow(dead_code)]
/// The Scheduler struct handles the scheduling of tasks on processors.
pub struct Scheduler {
    task_set: TaskSet,             // The set of tasks to schedule
    version: EDFVersion,           // The version of EDF to use (e.g., Partitioned, Global, EDFk)
    num_cores: usize,              // Number of processor cores
    num_threads: usize,            // Number of threads to use
    cores: Vec<Core>,              // The cores that handle task execution
    heuristic: String,             // Heuristic for task partitioning (e.g., ff, nd)
    sorting_order: String,         // Sorting order for tasks (e.g., du, iu)
}

impl Scheduler {
    /// Creates a new scheduler instance.
    pub fn new(task_set: TaskSet, num_cores: usize, num_threads: usize, heuristic: String, sorting_order: String) -> Self {
        Self {
            task_set,
            version: EDFVersion::Partitioned, // Default to Partitioned EDF
            num_cores,
            num_threads,
            cores: (1..=num_cores).map(|id| Core::new(id as ID)).collect(),
            heuristic,
            sorting_order,
        }
    }

    /// Sets the version of EDF to use (e.g., Global, Partitioned, EDFk).
    pub fn set_version(&mut self, new_version: EDFVersion) {
        self.version = new_version;
    }

    /// Returns the k of EDF(k), or 1 if the version is not EDF(k).
    pub fn get_k(&self) -> usize {
        match self.version {
            EDFVersion::EDFk(value) => value,
            _ => 1, // Default to EDF(1) for non-EDF(k) versions
        }
    }

    /// Checks if the task set is schedulable under global EDF.
    pub fn check_global_edf_schedulability(&self) -> bool {
        let num_cores_f64 = self.num_cores as f64;
        self.task_set.utilisation() <= num_cores_f64 - (num_cores_f64 - 1.0) * self.task_set.max_utilisation()
    }

    /// Checks if the task set is schedulable under EDF(k).
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

    /// Partitions tasks over processors based on the selected heuristic and sorting order.
    pub fn partition_tasks(&mut self) -> Vec<TaskSet> {
        // Sort tasks based on utilization, either descending ("du") or ascending ("iu")
        if self.sorting_order == "du" {
            self.task_set.get_tasks_mut().sort_by(|a, b| b.utilisation().partial_cmp(&a.utilisation()).unwrap_or(Ordering::Equal));
        } else if self.sorting_order == "iu" {
            self.task_set.get_tasks_mut().sort_by(|a, b| a.utilisation().partial_cmp(&b.utilisation()).unwrap_or(Ordering::Equal));
        } else {
            panic!("Unknown sorting order");
        }

        let mut partitions: Vec<TaskSet> = vec![TaskSet::new_empty(); self.num_cores]; // Initialize empty partitions
        let num_task = self.task_set.get_tasks_mut().len();

        match self.heuristic.as_str() {
            "ff" => { // First-fit heuristic
                for task in self.task_set.get_tasks_mut().iter() {
                    for task_set in partitions.iter_mut() {
                        if task_set.iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() <= 1.0 {
                            task_set.add_task(task.clone());
                            break;
                        }
                    }
                }
            }
            "nd" => { // Next-fit heuristic
                let mut current_partition = 0;
                for task in self.task_set.get_tasks_mut().iter() {
                    if partitions[current_partition].iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() > 1.0 {
                        current_partition = (current_partition + 1) % self.num_cores;
                    }
                    partitions[current_partition].add_task(task.clone());
                }
            }
            "bf" => { // Best-fit heuristic
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
            "wf" => { // Worst-fit heuristic
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

        let num_task_computed: usize = partitions.iter().map(|taskset| taskset.len()).sum();

        if num_task > num_task_computed {
            eprintln!("Too many tasks ({:?}, {:?} attached to a processor) for {:?} processors", num_task, num_task_computed, self.num_cores);
            println!("{:#?}", partitions);
            exit(SchedulingCode::UnschedulableShortcut as i32)
        }

        partitions
    }

    /// Main function for partitioned EDF scheduling.
    pub fn compute_partitionned(&mut self) -> SchedulingCode {
        let mut processor_done = 0;
        let partition = self.partition_tasks(); // Partition tasks across processors

        // Assign each partitioned task set to a core
        self.cores = (1..=self.num_cores)
            .map(|id| Core::new_task_set(id as ID, partition[id - 1].clone()))
            .collect();

        println!("Cores : {:#?}", self.cores);

        let mut result = SchedulingCode::SchedulableShortcut;

        while processor_done < self.num_cores {
            let current_core = self.cores.get_mut(processor_done).unwrap();
            let resp = current_core.simulate_partitionned(); // Simulate partitioned execution

            if !(resp == SchedulingCode::SchedulableShortcut || resp == SchedulingCode::SchedulableSimulated) {
                println!("Taskset not schedulable");
                return resp; // If any core is not schedulable, return the result
            }

            if resp == SchedulingCode::SchedulableSimulated {
                result = resp; // Update the result if a core has been simulated
            }

            processor_done += 1;
        }

        result // Return the result after all cores are processed
    }

    /// Schedules jobs using EDFk (Earliest Deadline First with k processors).
    pub fn schedule_jobs(queue: &Vec<Job>, _: usize) -> Option<ID> {
        if queue.len() == 0 {
            return None;
        }

        let mut smallest = TimeStep::MAX;
        let mut index_to_ret: Option<ID> = None;

        for i in 0..queue.len() {
            if smallest == TimeStep::MIN {
                break;
            }

            let job = &queue[i];

            let current_deadline = job.absolute_deadline();
            if current_deadline < smallest {
                smallest = current_deadline;
                index_to_ret = Some(i as ID);
            }
        }

        index_to_ret
    }

    /// Computes scheduling for EDFk.
    pub fn compute_edfk(&mut self) -> SchedulingCode {
        if !self.check_edf_k_schedulability() {
            return SchedulingCode::UnschedulableShortcut; // If not schedulable, return shortcut
        }

        let k = self.get_k();
        let mut queue: Vec<(Job, ID)> = Vec::new();
        let mut result = SchedulingCode::SchedulableSimulated;

        let (mut time, max_time) = self.task_set.feasibility_interval();

        while time < max_time {
            let mut job_to_add = self.task_set.release_jobs(time);

            // Set the jobs deadline for task_ids < k to -inf (ignore their deadlines)
            for job in job_to_add.iter_mut() {
                if job.task_id() < k as ID {
                    job.set_deadline_inf();
                }
            }

            queue.extend(job_to_add.into_iter().map(|job| (job, 0)));

            let job_queue: Vec<Job> = queue.iter().map(|(job, _)| job.clone()).collect();
            let index = Self::schedule_jobs(&job_queue, k); // Schedule the job based on EDFk

            if let Some(index) = index {
                let (job, _) = &queue[index as usize];
                let job_time = job.absolute_deadline();

                if time == job_time {
                    queue.push((job.clone(), index));
                    result = SchedulingCode::SchedulableSimulated;
                }
            }

            time += 1;
        }

        result
    }

    /// Computes global EDF scheduling.
    pub fn compute_global(&mut self) -> SchedulingCode {
        if !self.check_global_edf_schedulability() {
            return SchedulingCode::UnschedulableSimulated; // If not schedulable, return unschedulable
        }

        let mut queue: Vec<Job> = Vec::new();
        let mut result = SchedulingCode::SchedulableSimulated;
        let (mut time, max_time) = self.task_set.feasibility_interval();

        while time < max_time {
            let job_to_add = self.task_set.release_jobs(time);
            queue.extend(job_to_add);

            let index = Self::schedule_jobs(&queue, 1); // Use EDF for global scheduling

            if let Some(index) = index {
                let job = &queue[index as usize];
                if time == job.absolute_deadline() {
                    queue.push(job.clone());
                    result = SchedulingCode::SchedulableSimulated;
                }
            }

            time += 1;
        }

        result
    }

    /// Main function to test the task set using the selected EDF version.
    pub fn test_task_set(&mut self) -> SchedulingCode {
        match self.version {
            EDFVersion::Global => self.compute_global(),
            EDFVersion::Partitioned => self.compute_partitionned(),
            EDFVersion::EDFk(_) => self.compute_edfk(),
        }
    }
}