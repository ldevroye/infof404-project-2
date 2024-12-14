use std::ops::{Index, IndexMut};

use crate::TaskSet;


#[derive(Debug)]
pub struct Partition {
    partition: Vec<TaskSet>
}

/// Used to access Partition[index] -> returns self.partition[index]
impl Index<usize> for Partition {
    type Output = TaskSet;

    fn index(&self, index: usize) -> &Self::Output {
        // Return a reference to the n-th element in the partition
        &self.partition[index]
    }
}

/// Used to access mut Partition[index] -> returns mut self.partition[index]
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

    /// iter_mut over partition
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TaskSet> { 
        self.partition.iter_mut() 
    }

    /// iter over partition
    pub fn iter(&self) -> impl Iterator<Item = &TaskSet> { 
        self.partition.iter()  
    }

}
