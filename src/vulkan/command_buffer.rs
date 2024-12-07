use super::{*, core::*};
use ash::vk;

pub struct CommandBuffer {
    command_buffer: vk::CommandBuffer,
    cleanup_list: Vec<(vk::Buffer, vk_mem::Allocation)>
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
            cleanup_list: Vec::new()
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

    pub fn submit(device: &Device, command_buffers: &[CommandBuffer], wait_semaphores: &[(Semaphore, vk::PipelineStageFlags)], signal_semaphores: &[Semaphore],
     fence: &Fence){
        let mut command_buffer_handles = Vec::<vk::CommandBuffer>::with_capacity(command_buffers.len()); 

        for buffer in command_buffers{
            command_buffer_handles.push(buffer.get_command_buffer());
        }

        let mut wait_semaphore_handles = Vec::<vk::Semaphore>::with_capacity(wait_semaphores.len());
        let mut wait_dst_stage_masks = Vec::<vk::PipelineStageFlags>::with_capacity(wait_semaphores.len());

        for semaphore in wait_semaphores{
            wait_semaphore_handles.push(semaphore.0.get_semaphore());
            wait_dst_stage_masks.push(semaphore.1);
        }

        let mut signal_semaphore_handles = Vec::<vk::Semaphore>::with_capacity(signal_semaphores.len());

        for semaphore in signal_semaphores{
            signal_semaphore_handles.push(semaphore.get_semaphore());
        }

        let submit_info = vk::SubmitInfo{
            s_type: vk::StructureType::SUBMIT_INFO,

            command_buffer_count: command_buffer_handles.len() as u32,
            p_command_buffers: command_buffer_handles.as_ptr(),

            wait_semaphore_count: signal_semaphore_handles.len() as u32,
            p_wait_semaphores: signal_semaphore_handles.as_ptr(),
            p_wait_dst_stage_mask: wait_dst_stage_masks.as_ptr(),

            signal_semaphore_count: signal_semaphore_handles.len() as u32,
            p_signal_semaphores: signal_semaphore_handles.as_ptr(),

            ..Default::default() 
        };

        unsafe{device.get_ash_device().queue_submit(device.get_queue(), std::slice::from_ref(&submit_info), fence.get_fence())}.expect("Failed to submit command buffers");
    }

    pub fn add_to_cleanup_list(&mut self, buffer: vk::Buffer, allocation: vk_mem::Allocation){
        self.cleanup_list.push((buffer, allocation));
    }

    pub fn cleanup(&mut self, device: &Device){
        for allocation in &mut self.cleanup_list{

            unsafe{
                device.get_allocator().destroy_buffer(allocation.0, &mut allocation.1);
            }
        }

        self.cleanup_list.clear();
    }

}
