use super::{*, core::*};
use ash::vk;

pub struct CommandBuffer {
    command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
    fn create_command_buffer(device: &Device, command_pool: &CommandPool) -> vk::CommandBuffer {
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: command_pool.get_command_pool(),
            command_buffer_count: 1,
            level: vk::CommandBufferLevel::PRIMARY,
            ..Default::default()
        };

        let command_buffer = unsafe {
            device
                .get_ash_device()
                .allocate_command_buffers(&alloc_info)
        }
        .expect("Failed to allocate the command buffers")[0];

        command_buffer
    }

    pub fn new(device: &Device, command_pool: &CommandPool) -> CommandBuffer {
        let command_buffer = CommandBuffer::create_command_buffer(device, command_pool);

        CommandBuffer {
            command_buffer: command_buffer,
        }
    }

    pub fn get_command_buffer(&self) -> vk::CommandBuffer{
        self.command_buffer
    }

    pub fn begin(&self, device: &Device){
        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe{device.get_ash_device().begin_command_buffer(self.command_buffer,
            &begin_info)}.expect("Failed to begin the command buffer");
    }

    pub fn end(&self, device: &Device){
        unsafe{device.get_ash_device().end_command_buffer(self.command_buffer)}
        .expect("Failed to end the command buffer");
    }


}
