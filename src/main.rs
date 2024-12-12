use std::error::Error;
use std::process;
use clap::{Arg, ArgMatches, Command};
use csv::ReaderBuilder;
use rayon::prelude::*;
use std::cmp::Ordering;

use multiprocessor::core::simulation;
use multiprocessor::models::{task::Task, taskset::TaskSet, scheduler::{EarliestDeadlineFirst}};
use multiprocessor::TimeStep;
use multiprocessor::constants::{EDFVersion, Heuristic, Sorting};


/// Reads a task set file and returns a `TaskSet`
pub fn read_task_file(file_path: &String) -> Result<TaskSet, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(false).from_path(file_path)?;
    let mut tasks = Vec::new();

    let mut id = 0;

    for result in rdr.records() {
        let record = result?;

        let offset: TimeStep = record[0].parse()?;
        let computation_time: TimeStep = record[1].trim().parse()?;
        let deadline: TimeStep = record[2].trim().parse()?;
        let period: TimeStep = record[3].trim().parse()?;

        tasks.push(Task::new(id, offset, computation_time, deadline, period));
        id += 1;
    }

    Ok(TaskSet::new(tasks))
}

fn parse_args() -> ArgMatches {
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
            )//.value_parser(["global", "partitioned", "k"]))
            
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

        .get_matches()
}


fn partition_tasks(tasks: &mut Vec<Task>, m: usize, heuristic: &str, order: &str) -> Vec<Vec<Task>> {
    if order == "du" {
        tasks.sort_by(|a, b| b.utilisation().partial_cmp(&a.utilisation()).unwrap_or(Ordering::Equal));
    } else {
        tasks.sort_by(|a, b| a.utilisation().partial_cmp(&b.utilisation()).unwrap_or(Ordering::Equal));
    }

    let mut partitions: Vec<Vec<Task>> = vec![Vec::new(); m];

    match heuristic {
        "ff" => {
            for task in tasks.iter() {
                for partition in partitions.iter_mut() {
                    if partition.iter().map(|t| t.utilisation()).sum::<f64>() + task.utilisation() <= 1.0 {
                        partition.push(task.clone());
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
                partitions[current_partition].push(task.clone());
            }
        }
        "bf" => {
            for task in tasks.iter() {
                let mut best_partition: Option<usize> = None;
                let mut min_slack = f64::MAX;

                for (i, partition) in partitions.iter().enumerate() {
                    let slack = 1.0 - partition.iter().map(|t| t.utilisation()).sum::<f64>();
                    if slack >= task.utilisation() && slack < min_slack {
                        best_partition = Some(i);
                        min_slack = slack;
                    }
                }

                if let Some(best) = best_partition {
                    partitions[best].push(task.clone());
                }
            }
        }
        "wf" => {
            for task in tasks.iter() {
                let mut worst_partition: Option<usize> = None;
                let mut max_slack = f64::MIN;

                for (i, partition) in partitions.iter().enumerate() {
                    let slack = 1.0 - partition.iter().map(|t| t.utilisation()).sum::<f64>();
                    if slack >= task.utilisation() && slack > max_slack {
                        worst_partition = Some(i);
                        max_slack = slack;
                    }
                }

                if let Some(worst) = worst_partition {
                    partitions[worst].push(task.clone());
                }
            }
        }
        _ => panic!("Unknown heuristic"),
    }

    partitions
}

fn main() {
    // cargo run <task_file> <m> -v global|partitioned|<k> [-w <w>] [-h ff|nf|bf|wf] [-s iu|du]
    // example : cargo run taskset.txt 4 -v partitioned -w 8 -h ff -s iu
    let matches:ArgMatches = parse_args();

    // Read tasks from file
    let taskset = match read_task_file(matches.get_one::<String>("task_file").unwrap()) {
        Ok(taskset) => taskset,
        Err(e) => {
            eprintln!("Error reading task file: {}", e);
            process::exit(2);
        }
    };

    let heuristic = matches.get_one::<String>("heuristic").unwrap();
    let core_number = matches.get_one::<String>("m").unwrap();
    let worker_number = matches.get_one::<String>("workers").unwrap();
    let version = matches.get_one::<String>("version").unwrap(); // TODO use cores, version heuristic & workers
    let sorting = matches.get_one::<String>("sorting").unwrap();


    let schedulable = simulation(taskset, 1);
        
    println!("{:?}", schedulable);

    process::exit(schedulable as i32);
}