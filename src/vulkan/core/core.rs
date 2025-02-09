use crate::Window;

use super::*;

use ash::Entry;

pub struct Core {
    entry: Entry,
    instance: Instance,
    device: Device,
    swapchain: Swapchain,
}

impl Core {
    pub fn new(window: &Window) -> Core {
        let entry = ash::Entry::linked();
        let instance = Instance::new(&entry, window);
        let mut device = Device::new(instance.get_ash_instance());
        let swapchain = Swapchain::new(&entry, window, &instance, &mut device);

        Core {
            entry,
            instance,
            device,
            swapchain,
        }
    }

    pub fn get_instance(&self) -> &Instance{
        &self.instance
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn get_device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    pub fn get_swapchain(&self) -> &Swapchain {
        &self.swapchain
    }
}
