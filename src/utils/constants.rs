#[derive(Debug, PartialEq)]
pub enum SchedulingCode {
    SchedulableSimulated = 0,
    SchedulableShortcut = 1,
    UnschedulableSimulated = 2,
    UnschedulableShortcut = 3,
    CannotTell = 4,
    Error = 5,
}


#[derive(Debug, PartialEq)]
pub enum EDFVersion {
    Global,
    Partitioned,
    EDFk(usize),
}

#[derive(Clone, Debug)]
pub enum Heuristic {
    FirstFit,
    NextFit,
    BestFit,
    WorstFit,
}

impl Heuristic {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ff" => Ok(Self::FirstFit),
            "nd" => Ok(Self::NextFit),
            "bf" => Ok(Self::BestFit),
            "wf" => Ok(Self::WorstFit),
            _ => Err(format!("Unknown heuristic: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Sorting {
    IncreasingUtilization,
    DecreasingUtilization,
}

impl Sorting {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "iu" => Ok(Self::IncreasingUtilization),
            "du" => Ok(Self::DecreasingUtilization),
            _ => Err(format!("Unknown sorting order: {}", s)),
        }
    }
}