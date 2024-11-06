use crate::Window;

use super::{instance::Instance, swapchain::Swapchain, device::Device};

use ash::Entry;

pub struct Core{
    entry: Entry,
    instance: Instance,
    device:  Device,
    swapchain: Swapchain
}

impl Core{
    pub fn new(window: &Window) -> Core{
        let entry = ash::Entry::linked();
        let instance = Instance::new(&entry, window);
        let mut device = Device::new(instance.get_ash_instance());
        let swapchain = Swapchain::new(&entry, window, &instance, &mut device);

        Core { entry: entry, instance: instance, device: device, swapchain: swapchain}
    }

    pub fn get_device(&self) -> &Device{
        &self.device
    }
}