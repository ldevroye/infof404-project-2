use std::error::Error;
use std::process;
use std::thread::available_parallelism;

use clap::{Command, Arg, ArgMatches};
use csv::ReaderBuilder;

use multiprocessor::constants::Version;
use multiprocessor::scheduler::Scheduler;


use multiprocessor::{Task, TaskSet, TimeStep};

/// Reads a task set file and returns a `TaskSet`.
/// 
/// # Arguments
/// * `file_path` - The path to the CSV file containing the task set.
/// 
/// # Returns
/// * `Result<TaskSet, Box<dyn Error>>` - A result that contains a TaskSet if successful, or an error if reading the file fails.
pub fn read_task_file(file_path: &String) -> Result<TaskSet, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(false).from_path(file_path)?;
    let mut tasks = Vec::new();

    let mut id = 1;

    for result in rdr.records() {
        let record = result?;

        // Parsing the task attributes
        let offset: TimeStep = record[0].parse()?;
        let computation_time: TimeStep = record[1].trim().parse()?;
        let deadline: TimeStep = record[2].trim().parse()?;
        let period: TimeStep = record[3].trim().parse()?;

        tasks.push(Task::new(id, offset, computation_time, deadline, period));
        id += 1;
    }

    Ok(TaskSet::new(tasks))
}

/// Builds the CLI command structure using `clap`.
/// 
/// # Returns
/// * `Command` - The configured CLI command with all arguments and options.
pub fn build_cli_command() -> Command {
    Command::new("EDF Scheduler")
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
            .help("Version of EDF to use (global, partitioned, or EDF(k))"))
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

fn main() {
    // CLI argument parsing and error handling
    let matches: ArgMatches = build_cli_command().get_matches();

    // Read tasks from the file
    let taskset = match read_task_file(matches.get_one::<String>("task_file").unwrap()) {
        Ok(taskset) => taskset,
        Err(e) => {
            eprintln!("Error reading task file: {}", e);
            process::exit(5);
        }
    };

    // Get system and CLI configuration
    let default_parallelism_approx = available_parallelism().unwrap().get();
    let heuristic = matches.get_one::<String>("heuristic").unwrap();
    let core_number = matches.get_one::<String>("m").unwrap().parse::<usize>().unwrap_or(1); // processors for the simulation
    let thread_number = matches.get_one::<String>("workers").unwrap().parse::<usize>().unwrap_or(default_parallelism_approx); // number of threads
    let version = matches.get_one::<String>("version").unwrap();
    let sorting = matches.get_one::<String>("sorting").unwrap();

    // Initialize the scheduler
    let mut scheduler: Scheduler = Scheduler::new(taskset, core_number, thread_number, heuristic.clone(), sorting.clone());

    // Set the scheduler version based on the argument
    match version.as_str() {
        "partitioned" => {
            scheduler.set_version(Version::Partitioned);
            println!("Partitioned");
        }
        "global" => {
            scheduler.set_version(Version::Global);
            println!("Schedule Tasks (Global EDF)");
        }
        "dm" => {
            scheduler.set_version(Version::GlobalDM);
            println!("Schedule Tasks (Global DM)");

        }
        _ => { // assume k is an integer if not the other two
            if let Ok(k) = version.parse::<usize>() {
                scheduler.set_version(Version::EDFk(k));
                println!("Schedule Tasks (EDF({:?}))", k);
            } else {
                eprintln!(
                    "Invalid version: '{}'. Please use 'partitioned', 'global', 'DM' or a valid integer for EDF(k).",
                    version
                );
                
                std::process::exit(5); // Optional: Exit the program with an error code
            }
        }
    }

    // Check if the task set is schedulable
    let schedulable = scheduler.test_task_set();
    println!("{:?}", schedulable);

    process::exit(schedulable as i32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_task_file;

    /// Test reading a valid task file
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

    /// Test reading an invalid task file
    #[test]
    fn test_read_task_file_invalid_format() {
        let task_file_content = "Invalid, Data";
        let file_path = "test_invalid.csv";

        std::fs::write(file_path, task_file_content).expect("Unable to write test file");

        let result = read_task_file(&file_path.to_string());
        assert!(result.is_err());

        std::fs::remove_file(file_path).expect("Failed to clean up test file");
    }

    /// Test the command line arguments parsing
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