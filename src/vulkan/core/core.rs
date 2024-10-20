use super::instance::Instance;

use ash::{Entry};

pub struct Core{
    entry: Entry,
    instance: Instance 
}

impl Core{
    pub fn new() -> Core{
        let entry = ash::Entry::linked();
        let instance = Instance::new(&entry);

        Core { entry: entry, instance: instance }
    }
}