use ash::vk;
use super::core::*;

pub struct Semaphore {
    semaphore: vk::Semaphore
}


impl Semaphore {
    pub fn new(device: &Device) -> Semaphore{
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe{device.get_ash_device().create_semaphore(&create_info, None)}
        .expect("Failed to create a semaphore");

        
        Semaphore {semaphore: semaphore}

    }

    pub fn get_semaphore(&self) -> vk::Semaphore{
       self.semaphore 
    }
}