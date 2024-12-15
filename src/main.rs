use std::error::Error;
use std::process;
use std::thread::available_parallelism;

use clap::{Arg, ArgMatches, Command};
use csv::ReaderBuilder;

use multiprocessor::constants::{Heuristic, Sorting};
use multiprocessor::scheduler::{Scheduler, PartitionScheduler};
use multiprocessor::{SchedulingCode, Task, TaskSet, TimeStep};

/// Reads a task set file and returns a `TaskSet`.
/// 
/// # Arguments
/// * `file_path` - Path to the task file.
///
/// # Returns
/// A `Result` containing a `TaskSet` if successful, or an error if parsing fails.
pub fn read_task_file(file_path: &str) -> Result<TaskSet, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(false).from_path(file_path)?;
    let mut tasks = Vec::new();
    let mut id = 1;

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

/// Builds the command-line interface using Clap.
///
/// # Returns
/// A `Command` that defines the CLI structure.
pub fn build_cli_command() -> Command {
    Command::new("EDF Scheduler")
        .disable_version_flag(true)
        .disable_help_flag(true)
        .arg(
            Arg::new("task_file")
                .required(true)
                .help("Path to the task set file"),
        )
        .arg(
            Arg::new("m")
                .required(true)
                .help("Number of cores"),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .required(true)
                .help("Version of EDF to use (global, partitioned, or EDF(k))"),
        )
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .help("Number of workers to run the simulation")
                .default_value("4"),
        )
        .arg(
            Arg::new("heuristic")
                .short('h')
                .long("heuristic")
                .help("Heuristic to use for partitioned scheduling")
                .value_parser(["ff", "nf", "bf", "wf"]),
        )
        .arg(
            Arg::new("sorting")
                .short('s')
                .long("sorting")
                .help("Task ordering based on utilization")
                .value_parser(["iu", "du"]),
        )
}

/// Entry point of the program.
fn main() {
    // Example usage: cargo run <task_file> <m> -v global|partitioned|<k> [-w <w>] [-h ff|nf|bf|wf] [-s iu|du]
    // Example: cargo run test.csv 4 -v partitioned -w 8 -h ff -s iu
    let matches: ArgMatches = build_cli_command().get_matches();

    // Read tasks from the specified file
    let taskset = match read_task_file(matches.get_one::<String>("task_file").unwrap()) {
        Ok(taskset) => taskset,
        Err(e) => {
            eprintln!("Error reading task file: {}", e);
            process::exit(SchedulingCode::Error as i32);
        }
    };

    // Determine default parallelism for the system
    let default_parallelism_approx = available_parallelism().unwrap().get();

    // Parse command-line arguments
    let core_number = matches
        .get_one::<String>("m")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(1);
    let version: &String = matches.get_one::<String>("version").unwrap();
    let thread_number = matches
        .get_one::<String>("workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(default_parallelism_approx);
    let heuristic = matches
        .get_one::<String>("heuristic")
        .map(|s| Heuristic::from_str(s).unwrap_or_else(|_| {
            eprintln!("Invalid heuristic: {}. Falling back to FirstFit.", s);
            Heuristic::FirstFit
        }))
        .unwrap_or(Heuristic::FirstFit);
    let sorting = matches
        .get_one::<String>("sorting")
        .map(|s| Sorting::from_str(s).unwrap_or_else(|_| {
            eprintln!("Invalid sorting: {}. Falling back to IncreasingUtilization.", s);
            Sorting::IncreasingUtilization
        }))
        .unwrap_or(Sorting::IncreasingUtilization);

    // Select scheduler type based on version
    let mut scheduler: Box<dyn Scheduler> = match version.as_str() {
        "partitioned" => {
            println!("Partitioned scheduling selected.");
            Box::new(PartitionScheduler::new(taskset, core_number, thread_number, heuristic, sorting))
        }
        "global" => {
            println!("Global EDF scheduling selected.");
            // Replace with actual global EDF scheduler initialization
            unimplemented!("Global EDF scheduler is not yet implemented.");        }
        _ => {
            println!("EDF(k) scheduling selected.");
            // Replace with actual EDF(k) scheduler initialization
            unimplemented!("EDF(k) scheduler is not yet implemented.");
        }
    };

    // Check if the task set is schedulable
    let schedulable = scheduler.is_schedulable();
    println!("Schedulability result: {:?}", schedulable);

    // Exit with the appropriate scheduling code
    process::exit(schedulable as i32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_task_file_valid() {
        let task_file_content = "\
            0, 20, 40, 50\n\
            10, 80, 200, 200";
        let file_path = "test_tasks.csv";

        std::fs::write(file_path, task_file_content).expect("Unable to write test file");

        let taskset = read_task_file(file_path).expect("Failed to read task set");
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

        let result = read_task_file(file_path);
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