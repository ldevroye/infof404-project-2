use crate::*;

/// Trait for scheduling algorithms.
pub trait Scheduler {
    /// Default check for schedulability. 
    /// Returns true by default.
    fn check_schedulability(&self) -> bool {
        true
    }

    /// Runs the simulation of the scheduling algorithm.
    fn run_simulation(&mut self) -> SchedulingCode;
}