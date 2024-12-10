use crate::models::TimeStep;
use gcd::Gcd;

fn lcm(a: TimeStep, b: TimeStep) -> TimeStep {
    (a * b) / a.gcd(b)
}

pub fn multiple_lcm(numbers: &[u32]) -> TimeStep {
    numbers.iter().fold(1, |acc, &x| lcm(acc, x))
}