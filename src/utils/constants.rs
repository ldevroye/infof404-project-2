pub type ID = usize;
pub type TimeStep = isize;

#[derive(PartialEq)]
pub enum CoreValue {
    Running,
    Complete,
    Missed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SchedulingCode {
    SchedulableSimulated = 0,
    SchedulableShortcut = 1,
    UnschedulableSimulated = 2,
    UnschedulableShortcut = 3,
    CannotTell = 4,
    Error = 5,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Version {
    GlobalEDF,
    PartitionEDF,
    EDFk(usize),
    GlobalDM,
}

impl Version {
    pub fn from_str(s: &str) -> Self {
        match s {
            "edf" => Version::GlobalEDF,
            "pedf" => Version::PartitionEDF,
            "edfk" => Version::EDFk(1),
            "dm" => Version::GlobalDM,
            _ => panic!("Invalid version string"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Heuristic {
    FirstFit,
    NextFit,
    BestFit,
    WorstFit,
}

impl Heuristic {
    pub fn from_str(s: &str) -> Self {
        match s {
            "first" => Heuristic::FirstFit,
            "next" => Heuristic::NextFit,
            "best" => Heuristic::BestFit,
            "worst" => Heuristic::WorstFit,
            _ => panic!("Invalid heuristic string"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Sorting {
    IncreasingUtilization,
    DecreasingUtilization,
}

impl Sorting {
    pub fn from_str(s: &str) -> Self {
        match s {
            "increasing" => Sorting::IncreasingUtilization,
            "decreasing" => Sorting::DecreasingUtilization,
            _ => panic!("Invalid sorting string"),
        }
    }
}