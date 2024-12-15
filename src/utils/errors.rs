use crate::models::{Job, TimeStep};

#[derive(Debug)]
pub enum SchedulingError {
    DeadlineMissed { job: Job, t: TimeStep },
}
