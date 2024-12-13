use crate::TaskSet;

pub struct Partition {
    partition: Vec<TaskSet>
}

impl Partition {

    pub fn new(processor_number: usize) -> Self {
        Self {
            partition: vec![TaskSet::new_empty(); processor_number],
        }
    }


    pub fn get(&mut self, index: usize) -> Option<&mut TaskSet> {
        if index < 1 || index > self.partition.len() { () }

        self.partition.get_mut(index)
    }

    

    pub fn processor_number(&self) -> usize {
        self.partition.len()
    }

}