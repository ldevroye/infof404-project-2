use std::path::PathBuf;
use std::error::Error;
use std::process;
use clap::Parser;
use csv;

use infof404_project_1_v2::core::simulation;
use infof404_project_1_v2::models::{task::Task, taskset::TaskSet, scheduler::{DeadlineMonotonic, EarliestDeadlineFirst, RoundRobin}};

#[derive(Parser, Debug)]
#[command(version, about = "Task Scheduler")]
struct Args {
    /// Scheduling algorithm to use: dm, edf, or rr
    #[arg(value_parser)]
    algorithm: String,

    /// Path to the task set file
    #[arg(value_parser)]
    taskset_path: PathBuf,
}

/// Reads a task set file and returns a `TaskSet`
pub fn read_task_file(file_path: &str) -> Result<TaskSet, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut tasks = Vec::new();

    let mut id = 0;

    for result in rdr.records() {
        let record = result?;
        let offset: u32 = record[0].parse()?;
        let computation_time: u32 = record[1].parse()?;
        let deadline: u32 = record[2].parse()?;
        let period: u32 = record[3].parse()?;

        tasks.push(Task::new(id, offset, computation_time, deadline, period));
        id += 1;
    }

    Ok(TaskSet::new(tasks))
}

fn main() {
    let args = Args::parse();

    // Read tasks from file
    let taskset = match read_task_file(args.taskset_path.to_str().unwrap()) {
        Ok(taskset) => taskset,
        Err(e) => {
            eprintln!("Error reading task file: {}", e);
            process::exit(2);
        }
    };

    // Choose the scheduling algorithm
    let schedulable = match args.algorithm.as_str() {
        "dm" => {
            let mut scheduler = DeadlineMonotonic;
            simulation(taskset, &mut scheduler)
        },
        "edf" => {
            let mut scheduler = EarliestDeadlineFirst;
            simulation(taskset, &mut scheduler)
        },
        "rr" => {
            let mut scheduler = RoundRobin::new();
            simulation(taskset, &mut scheduler)
        },
        _ => {
            eprintln!("Invalid algorithm specified. Choose dm, edf, or rr.");
            process::exit(3);
        }
    };

    process::exit(schedulable as i32);
}