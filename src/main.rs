use std::str::FromStr;
use std::error::Error;
use std::process;
use clap::{Parser, ValueEnum};
use csv;

use infof404_project_1_v2::core::simulation;
use infof404_project_1_v2::models::{task::Task, taskset::TaskSet, scheduler::{EarliestDeadlineFirst}};
use infof404_project_1_v2::TimeStep;

/// Distribution type
/// This is not a ValueEnum because of K(String) being dynamic
#[derive(Debug)]
enum Distribution {
    Global,
    Partitioned,
    K(u32),
}

impl FromStr for Distribution {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "global" => Ok(Distribution::Global),
            "partitioned" => Ok(Distribution::Partitioned),
            other => Ok(Distribution::K(other.trim().parse().unwrap())),
        }
    }
}

/// Heuristic options
#[derive(Debug, Clone, ValueEnum)]
enum Heuristic {
    Ff,
    Nf,
    Bf,
    Wf,
}

/// Scheduling strategies
#[derive(Debug, Clone, ValueEnum)]
enum Scheduling {
    Iu,
    Du,
}

#[derive(Parser, Debug)]
#[command(
    name = "Cargo Run Task Parser",
    about = "Parses task file and configuration for running tasks",
    disable_help_flag = true
)]
struct Args {
    /// Path to the task file
    task_file: String,

    /// Specifies the m value
    m: String,

    #[arg(short, long)]
    version_flag: bool,

    /// Specifies the type of distribution
    distribution: String,

    /// Specifies the weight parameter
    #[arg(short, long)]
    weight: Option<u32>,

    /// Specifies the heuristic strategy
    #[arg(short, long)]
    heuristic: Option<Heuristic>,

    /// Specifies the scheduling strategy
    #[arg(short, long)]
    scheduling: Option<Scheduling>,

}


/// Reads a task set file and returns a `TaskSet`
pub fn read_task_file(file_path: String) -> Result<TaskSet, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut tasks = Vec::new();

    let mut id = 0;

    for result in rdr.records() {
        let record = result?;
        let offset: TimeStep = record[0].parse()?;
        let computation_time: TimeStep = record[1].trim().parse()?;;
        let deadline: TimeStep = record[2].trim().parse()?;
        let period: TimeStep = record[3].trim().parse()?;

        tasks.push(Task::new(id, offset, computation_time, deadline, period));
        id += 1;
    }

    Ok(TaskSet::new(tasks))
}

fn main() {
    let args = Args::try_parse().unwrap_or_else(|err| {
        eprintln!("{}\n", err);
        std::process::exit(1);
    });

    /* 
    println!("Task File: {}", args.task_file);
    println!("M: {}", args.m);
    println!("Distribution: {}", args.distribution);

    if let Some(weight) = args.weight {
        println!("Weight: {}", weight);
    }
    if let Some(heuristic) = args.heuristic {
        println!("Heuristic: {:?}", heuristic);
    }
    if let Some(scheduling) = args.scheduling {
        println!("Scheduling: {:?}", scheduling);
    }
    */
    
    // Read tasks from file
    let taskset = match read_task_file(args.task_file) {
        Ok(taskset) => taskset,
        Err(e) => {
            eprintln!("Error reading task file: {}", e);
            process::exit(2);
        }
    };

    let mut scheduler = EarliestDeadlineFirst;
    let schedulable = simulation(taskset, &mut scheduler);
        
    

    process::exit(schedulable as i32);
}