#[derive(Debug)]
pub enum SchedulingCode {
    SchedulableSimulated = 0,
    UnschedulableSimulated = 1,
    SchedulableShortcut = 2,
    UnschedulableShortcut = 3,
}