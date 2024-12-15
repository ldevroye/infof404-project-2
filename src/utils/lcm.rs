use std::cmp::min;
use std::iter::Iterator;

// Helper function to calculate the GCD
fn gcd(a: isize, b: isize) -> isize {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}

// Function to calculate the LCM of two numbers
fn lcm(a: isize, b: isize) -> isize {
    if a == 0 || b == 0 {
        0 // LCM is undefined for zero
    } else {
        (a.abs() / gcd(a, b)) * b.abs()
    }
}

// Function to calculate the LCM of a vector of numbers
pub fn multiple_lcm(numbers: Vec<isize>) -> isize {
    numbers.into_iter().reduce(|acc, n| lcm(acc, n)).unwrap_or(0)
}