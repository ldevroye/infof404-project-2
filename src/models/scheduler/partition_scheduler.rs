use crate::constants::{Heuristic, Sorting};
use crate::{SchedulingCode, TaskSet, ID};
use crate::scheduler::core::Core;
use crate::scheduler::Scheduler;
use std::cmp::Ordering;

/// A scheduler for partitioned EDF scheduling
#[allow(dead_code)]
pub struct PartitionScheduler {
    task_set: TaskSet,
    num_cores: usize,
    num_threads: usize,
    cores: Vec<Core>,
    heuristic: Heuristic,
    sorting_order: Sorting,
}

impl PartitionScheduler {
    /// Creates a new instance of `PartitionScheduler`
    pub fn new(
        task_set: TaskSet,
        num_cores: usize,
        num_threads: usize,
        heuristic: Heuristic,
        sorting_order: Sorting,
    ) -> Self {
        Self {
            task_set,
            num_cores,
            num_threads,
            cores: (1..=num_cores).map(|id| Core::new(id as ID)).collect(),
            heuristic,
            sorting_order,
        }
    }
}

impl Scheduler for PartitionScheduler {
    fn partition_tasks(&mut self) -> Vec<TaskSet> {
        match self.sorting_order {
            Sorting::DecreasingUtilization => {
                self.task_set.get_tasks_mut().sort_by(|a, b| {
                    b.utilisation().partial_cmp(&a.utilisation()).unwrap_or(Ordering::Equal)
                });
            }
            Sorting::IncreasingUtilization => {
                self.task_set.get_tasks_mut().sort_by(|a, b| {
                    a.utilisation().partial_cmp(&b.utilisation()).unwrap_or(Ordering::Equal)
                });
            }
        }

        let mut partitions = vec![TaskSet::new_empty(); self.num_cores];
        match self.heuristic {
            Heuristic::FirstFit => {
                for task in self.task_set.get_tasks_mut().iter() {
                    for partition in partitions.iter_mut() {
                        if partition.iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() <= 1.0 {
                            partition.add_task(task.clone());
                            break;
                        }
                    }
                }
            }
            Heuristic::NextFit => {
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
            Heuristic::BestFit => {
                for task in self.task_set.get_tasks_mut().iter() {
                    let mut best_partition = None;
                    let mut min_slack = f64::MAX;

                    for (i, partition) in partitions.iter().enumerate() {
                        let slack = 1.0 - partition.iter().map(|t| t.utilisation()).sum::<f64>();
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
            Heuristic::WorstFit => {
                for task in self.task_set.get_tasks_mut().iter() {
                    let mut worst_partition = None;
                    let mut max_slack = f64::MIN;

                    for (i, partition) in partitions.iter().enumerate() {
                        let slack = 1.0 - partition.iter().map(|t| t.utilisation()).sum::<f64>();
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
        }

        partitions
    }

    fn is_schedulable(&mut self) -> SchedulingCode {
        let partitions = self.partition_tasks();

        self.cores = (1..=self.num_cores)
            .map(|id| Core::new_task_set(id as ID, partitions[id - 1].clone()))
            .collect();

        let mut result = SchedulingCode::SchedulableShortcut;

        for core in self.cores.iter_mut() {
            let resp = core.simulate_partitionned();
            if resp != SchedulingCode::SchedulableShortcut && resp != SchedulingCode::SchedulableSimulated {
                return resp;
            }
            if resp == SchedulingCode::SchedulableSimulated {
                result = resp;
            }
        }

        result
    }
}