use std::thread::available_parallelism;
use std::error::Error;
use std::process;
use clap::{Command, Arg, ArgMatches};
use csv::ReaderBuilder;

use multiprocessor::constants::{Heuristic, Sorting, TimeStep, Version, SchedulingCode};
use multiprocessor::scheduler::*;
use multiprocessor::{Task, TaskSet};

/// Reads a task set file and returns a `TaskSet`.
/// 
/// # Arguments
/// * `file_path` - The path to the CSV file containing the task set.
/// 
/// # Returns
/// * `Result<TaskSet, Box<dyn Error>>` - A result containing the `TaskSet` if successful, or an error if reading the file fails.
pub fn read_task_file(file_path: &String) -> Result<TaskSet, Box<dyn Error>> {
    // Open the CSV file for reading without headers.
    let mut rdr = ReaderBuilder::new().has_headers(false).from_path(file_path)?;
    let mut tasks = Vec::new();
    let mut id = 1; // Task ID counter

    // Read each record (line) in the CSV file.
    for result in rdr.records() {
        let record = result?;

        // Parse task attributes: offset, computation time, deadline, and period.
        let offset: TimeStep = record[0].parse()?;
        let computation_time: TimeStep = record[1].trim().parse()?;
        let deadline: TimeStep = record[2].trim().parse()?;
        let period: TimeStep = record[3].trim().parse()?;

        // Create a new task and add it to the task list.
        tasks.push(Task::new(id, offset, computation_time, deadline, period));
        id += 1; // Increment task ID for the next task
    }

    // Return the task set containing all tasks.
    Ok(TaskSet::new(tasks))
}

/// Builds the CLI command structure using `clap`.
/// 
/// # Returns
/// * `Command` - The configured CLI command with all arguments and options.
pub fn build_cli_command() -> Command {
    Command::new("EDF Scheduler")
        .disable_version_flag(true) // Disable version flag for simplicity
        .disable_help_flag(true) // Disable help flag to enforce argument handling
        .arg(Arg::new("task_file")
            .required(true) // Ensure task_file argument is provided
            .help("Path to the task set file"))
        .arg(Arg::new("m")
            .required(true) // Ensure m (number of cores) is provided
            .help("Number of cores"))
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .required(true) // Version is required to choose EDF version
            .help("Version of EDF to use (global, partitioned, or EDF(k))"))
        .arg(Arg::new("workers")
            .short('w')
            .long("workers")
            .help("Number of workers to run the simulation")
            .default_value("4")) // Default to 4 workers if not specified
        .arg(Arg::new("heuristic")
            .short('h')
            .long("heuristic")
            .help("Heuristic to use for partitioned scheduling")
            .value_parser(["ff", "nf", "bf", "wf"])) // Heuristic options for partitioned scheduling
        .arg(Arg::new("sorting")
            .short('s')
            .long("sorting")
            .help("Task ordering based on utilization")
            .value_parser(["iu", "du"])) // Sorting options for task ordering
}

fn main() {
    // Parse command-line arguments
    let matches: ArgMatches = build_cli_command().get_matches();

    // Attempt to read the task file, handle errors if file reading fails
    let taskset = match read_task_file(matches.get_one::<String>("task_file").unwrap()) {
        Ok(taskset) => taskset,
        Err(e) => {
            eprintln!("Error reading task file: {}", e);
            process::exit(SchedulingCode::Error as i32); // Exit with an error code if task file reading fails
        }
    };

    // Retrieve system and configuration settings from CLI arguments
    let default_parallelism_approx = available_parallelism().unwrap().get();
    let version = matches.get_one::<Version>("version").unwrap();
    let core_number = matches.get_one::<String>("m").unwrap().parse::<usize>().unwrap_or(1); // Default to 1 core if invalid input
    let thread_number = matches.get_one::<String>("workers").unwrap().parse::<usize>().unwrap_or(default_parallelism_approx); // Default to available parallelism if not specified
    let heuristic: &Heuristic = matches.get_one::<Heuristic>("heuristic").unwrap();
    let sorting = matches.get_one::<Sorting>("sorting").unwrap();

    // Create the scheduler and run the simulation based on the selected EDF version
    let mut scheduler: Box<dyn Scheduler>;

    match version {
        Version::GlobalEDF => {
            scheduler = Box::new(GlobalEDFScheduler::new(taskset, core_number));
        }
        Version::PartitionEDF => {
            scheduler = Box::new(PartitionEDFScheduler::new(taskset, core_number, thread_number, heuristic.clone(), sorting.clone()));
        }
        Version::EDFk(k) => {
            scheduler = Box::new(EDFkScheduler::new(taskset, core_number, *k));
        }
        Version::GlobalDM => {
            scheduler = Box::new(DMScheduler::new(taskset, core_number));
        }        
    }

    // Run the simulation and exit with the result
    let result = scheduler.run_simulation();
    process::exit(result as i32);
}