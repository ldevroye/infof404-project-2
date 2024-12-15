use std::ops::{Index, IndexMut};

use crate::TaskSet;

/// A partition represents a collection of task sets distributed across processors.
#[derive(Debug)]
pub struct Partition {
    partition: Vec<TaskSet>, // A vector of task sets, one for each processor
}

/// Implementing `Index` allows accessing the task set at a given index.
impl Index<usize> for Partition {
    type Output = TaskSet;

    fn index(&self, index: usize) -> &Self::Output {
        // Return a reference to the n-th element in the partition (task set for the processor)
        &self.partition[index]
    }
}

/// Implementing `IndexMut` allows mutable access to the task set at a given index.
impl IndexMut<usize> for Partition {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // Return a mutable reference to the n-th element in the partition
        &mut self.partition[index]
    }
}

impl Partition {
    /// Creates a new `Partition` with a specified number of processors.
    /// 
    /// # Arguments
    /// * `processor_number` - The number of processors in the partition.
    /// 
    /// # Returns
    /// * `Self` - A new `Partition` instance with empty task sets for each processor.
    pub fn new(processor_number: usize) -> Self {
        Self {
            partition: vec![TaskSet::new_empty(); processor_number],
        }
    }

    /// Retrieves a mutable reference to the task set at the given index.
    /// 
    /// # Arguments
    /// * `index` - The index of the processor to access.
    /// 
    /// # Returns
    /// * `Option<&mut TaskSet>` - A mutable reference to the task set at the index, or `None` if the index is out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut TaskSet> {
        if index < 1 || index > self.partition.len() {
            return None; // Index out of bounds
        }
        self.partition.get_mut(index)
    }

    /// Retrieves an immutable reference to the task set at the given index.
    /// 
    /// # Arguments
    /// * `index` - The index of the processor to access.
    /// 
    /// # Returns
    /// * `Option<&TaskSet>` - An immutable reference to the task set at the index, or `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&TaskSet> {
        if index < 1 || index > self.partition.len() {
            return None; // Index out of bounds
        }
        self.partition.get(index)
    }

    /// Returns the number of processors in the partition.
    /// 
    /// # Returns
    /// * `usize` - The number of processors (task sets) in the partition.
    pub fn processor_number(&self) -> usize {
        self.partition.len()
    }

    /// Returns a mutable iterator over the task sets in the partition.
    /// 
    /// # Returns
    /// * `impl Iterator<Item = &mut TaskSet>` - An iterator over mutable references to task sets.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TaskSet> { 
        self.partition.iter_mut() 
    }

    /// Returns an iterator over the task sets in the partition.
    /// 
    /// # Returns
    /// * `impl Iterator<Item = &TaskSet>` - An iterator over immutable references to task sets.
    pub fn iter(&self) -> impl Iterator<Item = &TaskSet> { 
        self.partition.iter()  
    }
}