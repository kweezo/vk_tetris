use ash::vk;

use super::core::*;

pub struct CommandPool {
    command_pool: vk::CommandPool,
}

impl CommandPool {
    fn create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
        let create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            queue_family_index: queue_family_index,
            ..Default::default()
        };

        let pool = unsafe {
            device
                .get_ash_device()
                .create_command_pool(&create_info, None)
        }
        .expect("Failed to create a command pool");

        pool
    }

    pub fn new(device: &Device, queue_family_index: u32) -> CommandPool {
        let pool = CommandPool::create_command_pool(&device, queue_family_index);

        CommandPool { command_pool: pool }
    }

    pub fn get_command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }
}
