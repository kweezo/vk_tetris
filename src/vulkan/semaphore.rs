use super::core::*;
use ash::vk;

pub struct Semaphore {
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &Device) -> Semaphore {
        let create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let semaphore = unsafe { device.get_ash_device().create_semaphore(&create_info, None) }
            .expect("Failed to create a semaphore");

        Semaphore { semaphore }
    }

    pub fn get_semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }
}
