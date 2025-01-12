use crate::{core::Device, types::*};
use ash::vk;
use bytemuck::bytes_of;
use game::text_renderer;
use crate::{descriptor::{DescriptorInfo, DescriptorSet}, *};
use std::pin::Pin;
use super::TextRenderer;

pub struct UserInterface {
    backdrop_tex: Texture,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    text_buffer: Buffer,

    text_renderer: TextRenderer
}

impl<'a> UserInterface {
    pub fn new(device: &Device, command_pool: &CommandPool, backdrop_path: &str) -> UserInterface {
        let text_renderer = TextRenderer::new(device, command_pool);
        let text_data = text_renderer.get_data_for_str("Hello, World!");

        let buffers = UserInterface::initialize_buffers(device, command_pool, backdrop_path, text_data);


        UserInterface {
            vertex_buffer: buffers.0,
            index_buffer: buffers.1,
            text_buffer: buffers.2,
            backdrop_tex: buffers.3,
            text_renderer
        }
    }

   

    fn initialize_buffers(
        device: &Device,
        command_pool: &CommandPool,
        backdrop_path: &str,
        text_data: Vec<u8>
    ) -> (Buffer, Buffer, Buffer, Texture) {
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

        let text_buffer = Buffer::new(
            device,
            &mut command_buffer,
            &text_data.as_slice(),
            BufferType::Vertex,
            false);


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

        (vertex_buffer, index_buffer, text_buffer, backdrop_texture)
    }

    pub fn draw(
        &self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
    ) {
        self.draw_backdrop(device, render_pass, command_buffer, subpass_index);

        self.text_renderer.prepare_text_renderer(device, command_buffer, &self.vertex_buffer, &self.index_buffer,
             render_pass, subpass_index+1);

        self.text_renderer.render_text(device, command_buffer, render_pass, &self.text_buffer, text_renderer::RenderInfo { char_count: 13, scale: 50f32,  pos: (10, 10) });
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
                &1i32.to_ne_bytes(),
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
            1
        );

        ([descriptor_write], infos)
    }

    pub fn get_text_renderer_descriptor_sets( 
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet; 2], Pin<Box<[DescriptorInfo; 2]>>) {
        self.text_renderer.get_descriptor_write_sets(set)
    }

    pub fn get_required_vertex_input_states() -> ([vk::PipelineVertexInputStateCreateInfo<'a>; 2], VertexInputData){
        let vertex_bindings = vec![
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 8,
                input_rate: vk::VertexInputRate::VERTEX,
            },

            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 1,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        ];

        let vertex_attributes = vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,

                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },

            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 1,

                format: vk::Format::R8_UINT,
                offset: 0,
            },
 
        ];

        ([
            vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,

                vertex_attribute_description_count: 1,
                p_vertex_attribute_descriptions: &vertex_attributes[0],

                vertex_binding_description_count: 1,
                p_vertex_binding_descriptions: &vertex_bindings[0],

                ..Default::default()
            },
        
            vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,

                vertex_attribute_description_count: vertex_attributes.len() as u32,
                p_vertex_attribute_descriptions: vertex_attributes.as_ptr(),

                vertex_binding_description_count: vertex_bindings.len() as u32,
                p_vertex_binding_descriptions: vertex_bindings.as_ptr(),

                ..Default::default()
            },
        ], vec![(vertex_bindings, vertex_attributes)])

    }

    pub fn destroy(&mut self, device: &Device) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
        self.backdrop_tex.destroy(device);
        self.text_renderer.destroy(device);
        self.text_buffer.destroy(device);
    }
}
