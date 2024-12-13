use std::error::Error;
use std::process;
use clap::{Command, Arg};
use csv::ReaderBuilder;
use clap::ArgMatches;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::thread::available_parallelism;

use multiprocessor::core::simulation;
use multiprocessor::{Task, TaskSet, TimeStep, Worker, ID, Partition};


/// Reads a task set file and returns a `TaskSet`
pub fn read_task_file(file_path: &String) -> Result<TaskSet, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(false).from_path(file_path)?;
    let mut tasks = Vec::new();

    let mut id = 0;

    for result in rdr.records() {
        let record = result?;
        println!("record : {:?}", record);

        let offset: TimeStep = record[0].parse()?;
        let computation_time: TimeStep = record[1].trim().parse()?;
        let deadline: TimeStep = record[2].trim().parse()?;
        let period: TimeStep = record[3].trim().parse()?;

        tasks.push(Task::new(id, offset, computation_time, deadline, period));
        id += 1;
    }

    Ok(TaskSet::new(tasks))
}

pub fn build_cli_command() -> Command {
    Command::new("EDF Scheduler")
    .version("1.0")
    .author("Your Name <your_email@example.com>")
    .about("Simulates EDF scheduling for task sets")
    .disable_version_flag(true)
    .disable_help_flag(true)
    
    .arg(Arg::new("task_file")
    .required(true)
    .help("Path to the task set file"))

    .arg(Arg::new("m")
        .required(true)
        .help("Number of cores"))

    .arg(Arg::new("version")
        .short('v')
        .long("version")
        .required(true)
        .help("Version of EDF to use (global, partitioned, or EDF(k))")
        .value_parser(["global", "partitioned", "k"]))
        
    .arg(Arg::new("workers")
        .short('w')
        .long("workers")
        .help("Number of workers to run the simulation")
        .default_value("4"))

    .arg(Arg::new("heuristic")
        .short('h')
        .long("heuristic")
        .help("Heuristic to use for partitioned scheduling")
        .value_parser(["ff", "nf", "bf", "wf"]))

    .arg(Arg::new("sorting")
        .short('s')
        .long("sorting")
        .help("Task ordering based on utilization")
        .value_parser(["iu", "du"]))
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
fn partition_tasks(tasks: &mut Vec<Task>, m: usize, heuristic: &str, order: &str) -> Partition {

    if order == "du" {
        tasks.sort_by(|a, b| b.utilisation().partial_cmp(&a.utilisation()).unwrap_or(Ordering::Equal));
    } else if order == "iu" {
        tasks.sort_by(|a, b| a.utilisation().partial_cmp(&b.utilisation()).unwrap_or(Ordering::Equal));
    } else {
        panic!("Unknown sorting order")
    }

    let mut partitions: Partition = Partition::new(m); // partition of each task per worker

    match heuristic {
        "ff" => {
            for task in tasks.iter() {
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
            for task in tasks.iter() {
                if partitions[current_partition]
                    .iter()
                    .map(|t| t.utilisation())
                    .sum::<f64>()
                    + task.utilisation()
                    > 1.0
                {
                    current_partition = (current_partition + 1) % m;
                }
                partitions[current_partition].add_task(task.clone());
            }
        }
        "bf" => {
            for task in tasks.iter() {
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
            for task in tasks.iter() {
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

    partitions
}


fn global_edf(tasks: &mut Vec<Task>, m: usize) -> Vec<Task> {
    tasks.sort_by(|a, b| a.deadline().cmp(&b.deadline()));
    tasks.iter().take(m).cloned().collect()
}

fn edf_k(tasks: &mut Vec<Task>, m: usize, k: usize) -> Vec<Task> {
    tasks.sort_by(|a, b| a.deadline().cmp(&b.deadline()));
    tasks.iter().take(k.min(m)).cloned().collect()
}


    
fn main() {
    // cargo run <task_file> <m> -v global|partitioned|<k> [-w <w>] [-h ff|nf|bf|wf] [-s iu|du]
    // example : cargo run test.csv 4 -v partitioned -w 8 -h ff -s iu
    let matches:ArgMatches = build_cli_command().get_matches();

    // Read tasks from file
    let mut taskset = match read_task_file(matches.get_one::<String>("task_file").unwrap()) {
        Ok(taskset) => taskset,
        Err(e) => {
            eprintln!("Error reading task file: {}", e);
            process::exit(2);
        }
    };
    let default_parallelism_approx = available_parallelism().unwrap().get();

    let heuristic = matches.get_one::<String>("heuristic").unwrap();
    let core_number = matches.get_one::<String>("m").unwrap().parse::<usize>().unwrap_or(1); // processors for the simulation
    let thread_number = matches.get_one::<String>("workers").unwrap().parse::<usize>().unwrap_or(default_parallelism_approx); // nbr threads
    let version = matches.get_one::<String>("version").unwrap();
    let sorting = matches.get_one::<String>("sorting").unwrap();

    let mut partitions: Partition = partition_tasks(taskset.get_tasks_mut(), core_number, heuristic, sorting);
    let workers: Vec<Worker> = (1..=core_number)
                            .map(|id| Worker::new(id as u32, HashMap::new()))
                            .collect();

                            
    match version.as_str() {
        "partitioned" => {
            let partitions = partition_tasks(taskset.get_tasks_mut(), core_number, heuristic, sorting);
            println!("Partitions: {:#?}", partitions);
        }
        "global" => {
            let scheduled_tasks = global_edf(taskset.get_tasks_mut(), core_number);
            println!("Scheduled Tasks (Global EDF): {:#?}", scheduled_tasks);
        }
        _ => { // assume k is an integer if not the other two
            let scheduled_tasks = edf_k(taskset.get_tasks_mut(), core_number, version.parse::<usize>().unwrap());
            println!("Scheduled Tasks (EDF({:?})): {:#?}", version, scheduled_tasks);
        }
    }

    println!("Taskset : {:#?}", taskset.get_tasks_mut());

    let schedulable = simulation(partitions, thread_number);
        
    println!("{:?}", schedulable);

    process::exit(schedulable as i32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_task_file;

    #[test]
    fn test_read_task_file_valid() {
        let task_file_content = "\
            0, 20, 40, 50\n\
            10, 80, 200, 200";
        let file_path = "test_tasks.csv";

        std::fs::write(file_path, task_file_content).expect("Unable to write test file");

        let taskset = read_task_file(&file_path.to_string()).expect("Failed to read task set");
        let task = &taskset.get_tasks()[0];

        assert_eq!(taskset.get_tasks().len(), 2);
        assert_eq!(task.offset(), 0);
        assert_eq!(task.wcet(), 20);
        assert_eq!(task.deadline(), 40);
        assert_eq!(task.period(), 50);

        std::fs::remove_file(file_path).expect("Failed to clean up test file");
    }

    #[test]
    fn test_read_task_file_invalid_format() {
        let task_file_content = "Invalid, Data";
        let file_path = "test_invalid.csv";

        std::fs::write(file_path, task_file_content).expect("Unable to write test file");

        let result = read_task_file(&file_path.to_string());
        assert!(result.is_err());

        std::fs::remove_file(file_path).expect("Failed to clean up test file");
    }

    #[test]
    fn test_command_line_arguments() {
        let matches = build_cli_command().try_get_matches_from(vec![
            "edf_scheduler",
            "tasks.csv",
            "4",
            "-v",
            "global",
            "-w",
            "8",
        ]);

        assert!(matches.is_ok());
        let matches = matches.unwrap();

        assert_eq!(
            matches.get_one::<String>("task_file").unwrap(),
            "tasks.csv"
        );
        assert_eq!(
            *matches.get_one::<String>("m").unwrap(),
            "4"
        );
        assert_eq!(
            matches.get_one::<String>("version").unwrap(),
            "global"
        );
        assert_eq!(
            *matches.get_one::<String>("workers").unwrap(),
            "8"
        );
    }
}