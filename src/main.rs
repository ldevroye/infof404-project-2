use std::error::Error;
use std::process;
use clap::{Command, Arg};
use csv::ReaderBuilder;

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

fn main() {
    // cargo run <task_file> <m> -v global|partitioned|<k> [-w <w>] [-h ff|nf|bf|wf] [-s iu|du]
    // example : cargo run taskset.txt 4 -v partitioned -w 8 -h ff -s iu
    let matches = Command::new("EDF Scheduler")
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

        .get_matches();
    
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

    println!("taskset : {:#?}", taskset);

    let schedulable = simulation(taskset, 1);
        
    print!("{:?}\n", schedulable);

    process::exit(schedulable as i32);
}