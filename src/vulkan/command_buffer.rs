use super::{core::*, *};
use ash::vk;

pub struct CommandBuffer {
    command_buffer: vk::CommandBuffer,
    cleanup_list: Vec<(vk::Buffer, vk_mem::Allocation)>,
}

impl CommandBuffer {
    fn create_command_buffer(
        device: &Device,
        command_pool: &CommandPool,
        is_secondary: bool,
    ) -> vk::CommandBuffer {
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: command_pool.get_command_pool(),
            command_buffer_count: 1,
            level: if !is_secondary {
                vk::CommandBufferLevel::PRIMARY
            } else {
                vk::CommandBufferLevel::SECONDARY
            },
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

    pub fn new(device: &Device, command_pool: &CommandPool, is_secondary: bool) -> CommandBuffer {
        let command_buffer =
            CommandBuffer::create_command_buffer(device, command_pool, is_secondary);

        CommandBuffer {
            command_buffer,
            cleanup_list: Vec::new(),
        }
    }

    pub fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub fn begin(
        &self,
        device: &Device,
        inheritance_info: &vk::CommandBufferInheritanceInfo,
        flags: vk::CommandBufferUsageFlags,
    ) {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags,
            p_inheritance_info: inheritance_info,

            ..Default::default()
        };

        unsafe {
            device
                .get_ash_device()
                .begin_command_buffer(self.command_buffer, &begin_info)
        }
        .expect("Failed to begin the command buffer");
    }

    pub fn end(&self, device: &Device) {
        unsafe {
            device
                .get_ash_device()
                .end_command_buffer(self.command_buffer)
        }
        .expect("Failed to end the command buffer");
    }

    pub fn submit(
        device: &Device,
        command_buffers: &[vk::CommandBuffer],
        wait_semaphores: &[(vk::Semaphore, vk::PipelineStageFlags)],
        signal_semaphores: &[vk::Semaphore],
        fence: vk::Fence,
    ) {
        let mut wait_semaphore_handles = Vec::<vk::Semaphore>::with_capacity(wait_semaphores.len());
        let mut wait_dst_stage_masks =
            Vec::<vk::PipelineStageFlags>::with_capacity(wait_semaphores.len());

        for semaphore in wait_semaphores {
            wait_semaphore_handles.push(semaphore.0);
            wait_dst_stage_masks.push(semaphore.1);
        }

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,

            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: command_buffers.as_ptr(),

            wait_semaphore_count: wait_semaphore_handles.len() as u32,
            p_wait_semaphores: wait_semaphore_handles.as_ptr(),
            p_wait_dst_stage_mask: wait_dst_stage_masks.as_ptr(),

            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),

            ..Default::default()
        };
        
        let submit_slice = std::slice::from_ref(&submit_info);

        let queue = device.get_queue();

        unsafe {
            device.get_ash_device().queue_wait_idle(queue).expect("Failed to fail to fail the thingamajig");

            device.get_ash_device().queue_submit(
                queue,
                &submit_slice,
                fence,
            )
        }
        .expect("Failed to submit command buffers");
    }

    pub fn add_to_cleanup_list(&mut self, buffer: vk::Buffer, allocation: vk_mem::Allocation) {
        self.cleanup_list.push((buffer, allocation));
    }

    pub fn cleanup(&mut self, device: &Device) {
        for allocation in &mut self.cleanup_list {
            unsafe {
                device
                    .get_allocator()
                    .destroy_buffer(allocation.0, &mut allocation.1);
            }
        }

        self.cleanup_list.clear();
    }

    pub fn cleanup_raw(&mut self, allocator: &vk_mem::Allocator) {
        for allocation in &mut self.cleanup_list {
            unsafe {
                allocator.destroy_buffer(allocation.0, &mut allocation.1);
            }
        }

        self.cleanup_list.clear();
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        assert!(
            self.cleanup_list.is_empty(),
            "Command buffer's ({:?}) cleanup list wasn't cleared during runtime",
            self.command_buffer
        )
    }
}
