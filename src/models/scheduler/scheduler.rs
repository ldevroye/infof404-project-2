use crate::{Job, SchedulingCode, TaskSet, TimeStep, ID};
use crate::constants::Version;
use crate::scheduler::Core;

use std::cmp::Ordering;
use std::process::exit;

pub struct Scheduler {
    task_set: TaskSet,
    version: Version,
    num_cores: usize,
    num_threads: usize,
    cores: Vec<Core>,
    heuristic: String,
    sorting_order: String,
}

impl Scheduler {
    /// Creates a new scheduler instance.
    pub fn new(task_set: TaskSet, num_cores: usize, num_threads: usize, heuristic: String, sorting_order: String) -> Self {
        Self {
            task_set,
            version: Version::Partitioned,
            num_cores,
            num_threads,
            cores: (1..=num_cores).map(|id| Core::new(id as ID)).collect(),
            heuristic,
            sorting_order,
        }
    }

    /// Sets a new version for the scheduler.
    pub fn set_version(&mut self, new_version: Version) {
        self.version = new_version;
    }

    /// Returns the 'k' value of EDF(k), or 1 if the version is not EDF(k).
    pub fn get_k(&self) -> usize {
        match self.version {
            Version::EDFk(value) => value,
            _ => 1, // EDF(1) is just EDF
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

        // If k is greater than or equal to task set size, it's not schedulable
        if k >= self.task_set.len() - 1 {
            return false;
        }

        // If any task has full utilization, it's not schedulable
        if self.task_set.get_task(k).unwrap().utilisation() == 1.0 {
            return false;
        }

        // Check if the task set fits under EDF(k)
        self.num_cores as f64 >= k as f64 + self.task_set.get_task_utilisation(k + 1).unwrap() / (1.0 - self.task_set.get_task_utilisation(k).unwrap())
    }

    /// Partitions tasks across processors based on the selected heuristic and sorting order.
    pub fn partition_tasks(&mut self) -> Vec<TaskSet> {
        // Sort tasks based on utilization in the specified order
        if self.sorting_order == "du" {
            self.task_set.get_tasks_mut().sort_by(|a, b| b.utilisation().partial_cmp(&a.utilisation()).unwrap_or(Ordering::Equal));
        } else if self.sorting_order == "iu" {
            self.task_set.get_tasks_mut().sort_by(|a, b| a.utilisation().partial_cmp(&b.utilisation()).unwrap_or(Ordering::Equal));
        } else {
            panic!("Unknown sorting order");
        }

        // Initialize empty partitions for each core
        let mut partitions: Vec<TaskSet> = vec![TaskSet::new_empty(); self.num_cores];
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

        // Check if the task set can fit into the partitions
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

        // Simulate the scheduling process for each core
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

        // Find the job with the earliest deadline
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
    
    /// Computes the task set among all the cores using EDF(k), with possible migrations between cores during simulation.
    pub fn compute_edfk(&mut self) -> SchedulingCode {
        // Check if EDF(k) scheduling is possible
        if !self.check_edf_k_schedulability() {
            return SchedulingCode::UnschedulableShortcut; // If not schedulable, return shortcut
        }

        let k = self.get_k();
        let mut queue: Vec<(Job, ID)> = Vec::new();
        let mut result = SchedulingCode::SchedulableSimulated;

        // Compute the feasibility interval for the task set
        let (mut time, max_time) = self.task_set.feasibility_interval_global();
        println!("interval : [{:?}, {:#?}]", time, max_time);
        let mut threash_hold = max_time / 100;

        // Set the initial time for all cores
        self.cores.iter_mut().for_each(|core| core.set_time(time));

        while time < max_time {
            let mut job_to_add = self.task_set.release_jobs(time);

            // Set the jobs' deadline for task migration
            if !job_to_add.is_empty() {
                for i in 0..job_to_add.len() {
                    job_to_add[i].set_deadline(time);
                }
            }

            // Handle migrations across cores during scheduling
            for (mut task, core_id) in job_to_add {
                let mut assignable_core = self.cores.get_mut(core_id as usize).unwrap();
                if !assignable_core.is_accepted(task) {
                    assignable_core = self.cores.get_mut(core_id as usize).unwrap();
                    if !assignable_core.accepted {
                        continue;
                    }
                    assignable_core.add_task(task);
                } else {
                    assignable_core.add_task(task);
                }
            }

            let mut response_code = SchedulingCode::SchedulableSimulated;

            // Check for task migrations across cores
            for mut job in self.cores.iter_mut() {
                if !job.processing {
                    continue;
                }

                let queue_new_jobs = Self::schedule_jobs(&job.task_queue, self.num_threads);
                if let Some(current_job_index) = queue_new_jobs {
                    let mut task = &job.task_queue[current_job_index];
                    job.processing = false;
                    job.add_task(task);
                    response_code = SchedulingCode::SchedulableSimulated
                }
            }

            result = response_code;
            time += threash_hold;
        }

        result // Return the final result
    }
}