use std::ops::{Index, IndexMut};
use crate::TaskSet;

/// A partition represents a collection of task sets distributed across processors.
#[derive(Debug)]
pub struct Partition {
    partition: Vec<TaskSet>, // A vector holding task sets, each corresponding to a processor
}

/// Implementing `Index` allows accessing the task set at a given index (read-only access).
impl Index<usize> for Partition {
    type Output = TaskSet;

    fn index(&self, index: usize) -> &Self::Output {
        // Return a reference to the task set for the processor at the given index.
        &self.partition[index]
    }
}

/// Implementing `IndexMut` allows mutable access to the task set at a given index.
impl IndexMut<usize> for Partition {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // Return a mutable reference to the task set for the processor at the given index.
        &mut self.partition[index]
    }
}

impl Partition {
    /// Creates a new `Partition` with a specified number of processors.
    /// Each processor starts with an empty task set.
    ///
    /// # Arguments
    /// * `processor_number` - The number of processors (task sets) in the partition.
    ///
    /// # Returns
    /// * `Self` - A new `Partition` instance containing empty task sets for each processor.
    pub fn new(processor_number: usize) -> Self {
        // Initialize a partition with `processor_number` empty task sets
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
        // Ensure the index is within bounds
        if index < 1 || index > self.partition.len() {
            return None; // Index out of bounds, return None
        }
        self.partition.get_mut(index) // Return the mutable reference to the task set
    }

    /// Retrieves an immutable reference to the task set at the given index.
    ///
    /// # Arguments
    /// * `index` - The index of the processor to access.
    ///
    /// # Returns
    /// * `Option<&TaskSet>` - An immutable reference to the task set at the index, or `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&TaskSet> {
        // Ensure the index is within bounds
        if index < 1 || index > self.partition.len() {
            return None; // Index out of bounds, return None
        }
        self.partition.get(index) // Return the immutable reference to the task set
    }

    /// Returns the number of processors (task sets) in the partition.
    ///
    /// # Returns
    /// * `usize` - The number of processors in the partition.
    pub fn processor_number(&self) -> usize {
        // Return the length of the partition, i.e., the number of processors
        self.partition.len()
    }

    /// Returns a mutable iterator over the task sets in the partition.
    ///
    /// # Returns
    /// * `impl Iterator<Item = &mut TaskSet>` - An iterator yielding mutable references to task sets.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TaskSet> { 
        // Return a mutable iterator over the partition's task sets
        self.partition.iter_mut() 
    }

    /// Returns an iterator over the task sets in the partition.
    ///
    /// # Returns
    /// * `impl Iterator<Item = &TaskSet>` - An iterator yielding immutable references to task sets.
    pub fn iter(&self) -> impl Iterator<Item = &TaskSet> { 
        // Return an immutable iterator over the partition's task sets
        self.partition.iter()  
    }
}