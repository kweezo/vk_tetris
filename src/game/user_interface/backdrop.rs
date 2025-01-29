use std::pin::Pin;

use descriptor::{DescriptorInfo, DescriptorSet};

use ash::vk;

use crate::*;

pub struct Backdrop {
    backdrop_tex: Texture
}

impl<'a> Backdrop {
    pub fn new(device: &Device, command_pool: &CommandPool, path: &str) -> Backdrop {
        let tex = Backdrop::load_tex(device, command_pool, path);

        Backdrop {backdrop_tex: tex}
    }

    fn load_tex(device: &Device, command_pool: &CommandPool, path: &str) -> Texture{

        let mut command_buffer = CommandBuffer::new(device, command_pool, false);

        command_buffer.begin(device, &vk::CommandBufferInheritanceInfo::default(), vk::CommandBufferUsageFlags::empty());

        let tex = Texture::new(path, device, &mut command_buffer)
            .expect("Failed to load the backdrop texture");

        command_buffer.end(device);

        let fence = Fence::new(device, false);

        CommandBuffer::submit(device, &[command_buffer.get_command_buffer()], &[], &[], fence.get_fence());

        unsafe {
            device
                .get_ash_device()
                .wait_for_fences(&[fence.get_fence()], true, u64::MAX)
                .expect(
                    "Failed to wait for the fence for the initialization of UI command buffers",
                );
        }

        command_buffer.cleanup(device);

        tex

    }


    pub fn draw(
        &self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
        tetromino_instance_count: u32,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer) {
        let offset = 0u32;

        let push_constants = [1u32.to_ne_bytes(), tetromino_instance_count.to_ne_bytes()].concat();

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
                &push_constants,
            );

            device.get_ash_device().cmd_bind_vertex_buffers(
                command_buffer.get_command_buffer(),
                0,
                &[vertex_buffer.get_buffer()],
                &[0],
            );

            device.get_ash_device().cmd_bind_index_buffer(
                command_buffer.get_command_buffer(),
                index_buffer.get_buffer(),
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
    ) -> ([vk::WriteDescriptorSet<'a>; 1], Vec<Pin<Box<DescriptorInfo>>>) {

        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.backdrop_tex.get_image_view(),
            sampler: self.backdrop_tex.get_sampler(),
        };

        let infos = vec![
            Pin::new(
                Box::new(
                    DescriptorInfo::Image(vec![image_info])
                ))];

        let descriptor_write = set.create_write_set(
            &infos[0],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            1,
            1,
            1
        );

        ([descriptor_write], infos)
    }

    pub fn destroy(&mut self, device: &Device) {
        self.backdrop_tex.destroy(device);
    }
}