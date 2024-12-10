use super::{job::Job, TimeStep, ID};

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    id: ID,
    offset: TimeStep,   // O_i : Offset.
    wcet: TimeStep,     // C_i : Worst-case execution time.
    deadline: TimeStep, // D_i : Relative deadline.
    period: TimeStep,   // T_i : Period.
    jobs_released: u32,
}

impl Task {
    pub fn new(
        id: u32,
        offset: TimeStep,
        wcet: TimeStep,
        deadline: TimeStep,
        period: TimeStep,
    ) -> Self {
        Self {
            id,
            offset,
            wcet,
            deadline,
            period,
            jobs_released: 0,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn offset(&self) -> TimeStep {
        self.offset
    }

    pub fn wcet(&self) -> TimeStep {
        self.wcet
    }

    pub fn deadline(&self) -> TimeStep {
        self.deadline + self.offset
    }

    pub fn period(&self) -> TimeStep {
        self.period + self.offset
    }

    pub fn utilisation(&self) -> f64 {
        self.wcet as f64 / self.period as f64
    }

    pub fn spawn_job(&mut self, current_time: TimeStep) -> Option<Job> {
        // Not yet released
        if current_time < self.offset {
            return None;
        }
        // Not a time at which a job should be released
        if (current_time - self.offset) % self.period != 0 {
            return None;
        }
        
        self.jobs_released += 1;
        
        Some(Job::new(self.jobs_released,
            self.id, 
            self.wcet,
            self.deadline(), 
            self.deadline() + current_time))
    }
}
