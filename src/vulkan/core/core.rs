use super::instance::Instance;
use super::device::Device;

use ash::{Entry};

pub struct Core{
    entry: Entry,
    instance: Instance,
    device:  Device
}

impl Core{
    pub fn new() -> Core{
        let entry = ash::Entry::linked();
        let instance = Instance::new(&entry);
        let device = Device::new(instance.get_vk_instance());

        Core { entry: entry, instance: instance, device: device}
    }
}