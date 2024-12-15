use crate::TimeStep;


#[derive(PartialEq)]
pub enum CoreValue {
    Running,
    Commplete,
    Missed,
}

#[derive(Debug, PartialEq)]
pub enum SchedulingCode {
    SchedulableSimulated = 0,
    SchedulableShortcut = 1,
    UnschedulableSimulated = 2,
    UnschedulableShortcut = 3,
    CannotTell = 4,
}


#[derive(Debug, PartialEq)]
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