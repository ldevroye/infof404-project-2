#[derive(PartialEq)]
pub enum CoreValue {
    Running,
    Complete,
    Missed,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SchedulingCode {
    SchedulableSimulated = 0,
    SchedulableShortcut = 1,
    UnschedulableSimulated = 2,
    UnschedulableShortcut = 3,
    CannotTell = 4,
}


#[derive(Debug, PartialEq)]
pub enum Version {
    Global,
    Partitioned,
    EDFk(usize),
    GlobalDM,
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