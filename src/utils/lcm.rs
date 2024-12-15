use crate::models::TimeStep;
use gcd::Gcd;

fn lcm(a: usize, b: usize) -> usize {
    (a * b) / a.gcd(b)
}

pub fn multiple_lcm(numbers: &[TimeStep]) -> TimeStep {
    numbers.iter().fold(1, |acc, &x| 
                            (lcm((acc as usize).try_into().unwrap(),
                                 (x as usize).try_into().unwrap()) as usize).try_into().unwrap())
}