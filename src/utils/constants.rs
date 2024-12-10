use std::str::FromStr;
use clap::ValueEnum;


#[derive(Debug)]
pub enum SchedulingCode {
    SchedulableSimulated = 0,
    UnschedulableSimulated = 1,
    SchedulableShortcut = 2,
    UnschedulableShortcut = 3,
}


#[derive(Debug)]
pub enum EDFVersion {
    Global,
    Partitioned,
    EDFk(usize),
}

#[derive(Debug)]
pub enum Heuristic {
    FirstFit,
    NextFit,
    BestFit,
    WorstFit,
}

#[derive(Debug)]
pub enum Sorting {
    IncreasingUtilization,
    DecreasingUtilization,
}