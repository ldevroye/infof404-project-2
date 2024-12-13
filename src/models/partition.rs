use std::collections::btree_map::IterMut;
use std::ops::{Index, IndexMut};

use crate::TaskSet;


#[derive(Debug)]
pub struct Partition {
    partition: Vec<TaskSet>
}

/// Used to access Partition[index] -> returns self.partition[index]
impl Index<usize> for Partition {
    type Output = TaskSet;  // The type returned when indexed

    fn index(&self, index: usize) -> &Self::Output {
        // Return a reference to the n-th element in the partition
        &self.partition[index]
    }
}

impl IndexMut<usize> for Partition {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.partition[index]
    }
}

impl Partition {

    pub fn new(processor_number: usize) -> Self {
        Self {
            partition: vec![TaskSet::new_empty(); processor_number],
        }
    }


    pub fn get_mut(&mut self, index: usize) -> Option<&mut TaskSet> {
        if index < 1 || index > self.partition.len() { () }

        self.partition.get_mut(index)
    }

    pub fn get(&mut self, index: usize) -> Option<&TaskSet> {
        if index < 1 || index > self.partition.len() { () }

        self.partition.get(index)
    }

    

    pub fn processor_number(&self) -> usize {
        self.partition.len()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TaskSet> { // iter_mut over partition
        self.partition.iter_mut() 
    }

    pub fn iter(&self) -> impl Iterator<Item = &TaskSet> { // iter over partition
        self.partition.iter()  
    }

    // Implementing the Index trait for Partition
    
}
