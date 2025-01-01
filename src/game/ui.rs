use crate::{core::Device, *};
use ash::vk;
use bytemuck::bytes_of;
use descriptor::{DescriptorInfo, DescriptorSet};
use std::pin::Pin;

pub struct UserInterface {
    backdrop_tex: Texture,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl<'a> UserInterface {
    pub fn new(device: &Device, command_pool: &CommandPool, backdrop_path: &str) -> UserInterface {
        let buffers = UserInterface::initialize_buffers(device, command_pool, backdrop_path);

        UserInterface {
            vertex_buffer: buffers.0,
            index_buffer: buffers.1,
            backdrop_tex: buffers.2,
        }
    }

    fn initialize_buffers(
        device: &Device,
        command_pool: &CommandPool,
        backdrop_path: &str,
    ) -> (Buffer, Buffer, Texture) {
        let indices: [u16; 6] = [0, 1, 2, 1, 2, 3];
        let vertices: [f32; 8] = [0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32];

        let mut command_buffer = CommandBuffer::new(device, command_pool, false);

        command_buffer.begin(
            device,
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        let vertex_buffer = Buffer::new(
            device,
            &mut command_buffer,
            bytes_of(&vertices),
            BufferType::Vertex,
            false,
        );
        let index_buffer = Buffer::new(
            device,
            &mut command_buffer,
            bytes_of(&indices),
            BufferType::Index,
            false,
        );

        let backdrop_texture = Texture::new(backdrop_path, device, &mut command_buffer)
            .expect("Failed to load the backdrop texture");

        command_buffer.end(device);

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: 0,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer.get_command_buffer(),
            signal_semaphore_count: 0,
            ..Default::default()
        };

        let fence = Fence::new(device, false);

        unsafe {
            device
                .get_ash_device()
                .queue_submit(device.get_queue(), &[submit_info], fence.get_fence())
                .expect("Failed to submit the queue for initialization of UI buffers");

            device
                .get_ash_device()
                .wait_for_fences(&[fence.get_fence()], true, u64::MAX)
                .expect(
                    "Failed to wait for the fence for the initialization of UI command buffers",
                );
        }

        command_buffer.cleanup(device);

        (vertex_buffer, index_buffer, backdrop_texture)
    }

    pub fn draw(
        &self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
    ) {
        self.draw_backdrop(device, render_pass, command_buffer, subpass_index);
    }

    pub fn draw_backdrop(
        &self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
    ) {
        let offset = 0u32;

        unsafe {
            device.get_ash_device().cmd_bind_pipeline(
                command_buffer.get_command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                render_pass.get_pipeline(subpass_index as usize),
            );

            device.get_ash_device().cmd_push_constants(
                command_buffer.get_command_buffer(),
                render_pass.get_layout(),
                vk::ShaderStageFlags::ALL,
                offset,
                &1i32.to_ne_bytes()
            );

            device.get_ash_device().cmd_bind_vertex_buffers(
                command_buffer.get_command_buffer(),
                0,
                &[self.vertex_buffer.get_buffer()],
                &[0],
            );

            device.get_ash_device().cmd_bind_index_buffer(
                command_buffer.get_command_buffer(),
                self.index_buffer.get_buffer(),
                0,
                vk::IndexType::UINT16,
            );

            device.get_ash_device().cmd_draw_indexed(
                command_buffer.get_command_buffer(),
                6,
                1,
                0,
                0,
                0,
            );
        }
    }

    pub fn get_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet; 1], Pin<Box<[DescriptorInfo; 1]>>) {
        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.backdrop_tex.get_image_view(),
            sampler: self.backdrop_tex.get_sampler(),
        };

        let infos = Pin::new(Box::new([DescriptorInfo::Image(vec![image_info])]));

        let descriptor_write = set.create_write_set(
            &infos.as_ref()[0],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            1,
            1,
        );

        ([descriptor_write], infos)
    }

    pub fn destroy(&mut self, device: &Device) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
        self.backdrop_tex.destroy(device);
    }
}
