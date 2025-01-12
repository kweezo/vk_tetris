use ash::vk;
use vk_mem::Alloc;

use super::*;

pub enum BufferType {
    Vertex,
    Index,
    Uniform,
}

pub struct Buffer {
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
    size: u64,

    persistent_staging_buffer: bool,
    staging_buffer: Option<(vk::Buffer, vk_mem::Allocation)>,
}

impl Buffer {
    pub fn create_buffer(
        device: &core::Device,
        size: usize,
        usage: vk::BufferUsageFlags,
        required_flags: vk::MemoryPropertyFlags,
        preferred_flags: vk::MemoryPropertyFlags,
        vma_flags: vk_mem::AllocationCreateFlags,
    ) -> (vk::Buffer, vk_mem::Allocation) {
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: size as u64,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            usage,
            ..Default::default()
        };

        let allocation_info = vk_mem::AllocationCreateInfo {
            flags: vma_flags,
            usage: vk_mem::MemoryUsage::Auto,
            preferred_flags,
            required_flags,
            memory_type_bits: 0,
            user_data: size,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            device
                .get_allocator()
                .create_buffer(&buffer_info, &allocation_info)
        }
        .expect("Failed to allocate a new buffer with VMA");

        (buffer, allocation)
    }

    pub fn setup_staging_buffer(
        device: &core::Device,
        data: &[u8],
    ) -> (vk::Buffer, vk_mem::Allocation) {
        let (buffer, mut allocation) = Buffer::create_buffer(
            device,
            data.len(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, /*because life is too short for this shit*/
            vk::MemoryPropertyFlags::empty(),
            vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        );

        unsafe {
            let ptr = device
                .get_allocator()
                .map_memory(&mut allocation)
                .expect("Failed to map device memory");

            std::ptr::copy(data.as_ptr(), ptr, data.len());
            device.get_allocator().unmap_memory(&mut allocation);
        }

        (buffer, allocation)
    }

    fn upload_data_to_buffer(
        device: &core::Device,
        buffer: vk::Buffer,
        staging_buffer: vk::Buffer,
        staging_allocation: vk_mem::Allocation,
        command_buffer: &mut CommandBuffer,
        data: &[u8],
        persistent_staging_buffer: bool,
    ) -> Option<vk_mem::Allocation> {
        let region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: data.len() as u64,
        };

        unsafe {
            device.get_ash_device().cmd_copy_buffer(
                command_buffer.get_command_buffer(),
                staging_buffer,
                buffer,
                &[region],
            );
        }

        if !persistent_staging_buffer {
            command_buffer.add_to_cleanup_list(staging_buffer, staging_allocation);
            return None;
        }

        Some(staging_allocation)
    }

    pub fn new(
        device: &core::Device,
        command_buffer: &mut CommandBuffer,
        data: &[u8],
        buffer_type: BufferType,
        persistent_staging_buffer: bool,
    ) -> Buffer {
        let buffer_usage = match buffer_type {
            BufferType::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
            BufferType::Index => vk::BufferUsageFlags::INDEX_BUFFER,
            BufferType::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
        };

        let (buffer, allocation) = Buffer::create_buffer(
            device,
            data.len(),
            buffer_usage | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::empty(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk_mem::AllocationCreateFlags::empty(),
        );

        let (staging_buffer, staging_allocation) = Buffer::setup_staging_buffer(device, data);

        match Buffer::upload_data_to_buffer(
            device,
            buffer,
            staging_buffer,
            staging_allocation,
            command_buffer,
            data,
            persistent_staging_buffer,
        ) {
            Some(e) => Buffer {
                buffer,
                allocation,
                size: data.len() as u64,
                persistent_staging_buffer,
                staging_buffer: Some((staging_buffer, e)),
            },
            None => Buffer {
                buffer,
                allocation,
                size: data.len() as u64,
                staging_buffer: None,
                persistent_staging_buffer,
            },
        }
    }

    pub fn update(
        &mut self,
        device: &core::Device,
        command_buffer: &mut CommandBuffer,
        data: &[u8],
    ) {
        if data.len() != self.size as usize {
            eprintln!("WARNING: Trying to update buffer but the data provided is a different size");
        }

        if self.persistent_staging_buffer {
            Buffer::upload_data_to_buffer(
                device,
                self.buffer,
                self.staging_buffer.as_ref().unwrap().0,
                self.staging_buffer.take().unwrap().1,
                command_buffer,
                data,
                true,
            );
            return;
        }

        let (staging_buffer, staging_allocation) = Buffer::setup_staging_buffer(device, data);

        Buffer::upload_data_to_buffer(
            device,
            self.buffer,
            staging_buffer,
            staging_allocation,
            command_buffer,
            data,
            false,
        );
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn destroy(&mut self, device: &core::Device) {
        if self.persistent_staging_buffer {
            unsafe {
                device.get_allocator().destroy_buffer(
                    self.staging_buffer.as_ref().unwrap().0,
                    &mut self.staging_buffer.as_mut().unwrap().1,
                )
            };
        }
        unsafe {
            device
                .get_allocator()
                .destroy_buffer(self.buffer, &mut self.allocation)
        };
    }
}
