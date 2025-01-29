use crate::{game::text::TextRenderer, types::*, *};
use ash::vk;
use types::VertexInputData;
use super::Button;


pub struct ButtonManager {
    instance_buffer: Option<Buffer>,
    instance_data: Vec<u8>,
    instance_count: u32,

    creation_command_buffer: CommandBuffer,
    update_command_buffer: CommandBuffer,
    fence: Fence
}


impl<'a> ButtonManager {

    pub fn new(device: &Device, command_pool: &CommandPool) -> ButtonManager {

        let creation_command_buffer = CommandBuffer::new(device, command_pool, false);
        let update_command_buffer = CommandBuffer::new(device, command_pool, false);
        let fence = Fence::new(device, false);

        ButtonManager { instance_buffer: None, instance_data: Vec::new(), instance_count: 0, creation_command_buffer, update_command_buffer, fence }
    }

    pub fn clear_data(&mut self) {
        self.instance_data.clear();
        self.instance_count = 0;
    }

    pub fn add_button(&mut self, button: &Button) {
        self.instance_data.extend_from_slice(&button.get_raw_data());
    }

    pub fn update(&mut self, device: &Device) {
        self.update_command_buffer.begin(device, &vk::CommandBufferInheritanceInfo::default(), vk::CommandBufferUsageFlags::empty());

        match &mut self.instance_buffer {
            Some(buff) => buff.update(device, &mut self.update_command_buffer, &self.instance_data),
            None => {
                self.instance_buffer = Some(Buffer::new(device, &mut self.update_command_buffer, &self.instance_data, BufferType::Vertex, false));
            }
        };

        self.instance_count = (self.instance_data.len() / 32) as u32;


        self.update_command_buffer.end(device);

        CommandBuffer::submit(device, &[self.update_command_buffer.get_command_buffer()], &[], &[], self.fence.get_fence());
         
        unsafe{
            device.get_ash_device().wait_for_fences(&[self.fence.get_fence()], true, std::u64::MAX)
             .expect("Failed to wait for the button creation fence.");

            device.get_ash_device().reset_fences(&[self.fence.get_fence()])
             .expect("Failed to reset the button manager creation fence");
        }

        self.update_command_buffer.cleanup(device);
    }

    pub fn draw(&self, device: &Device, command_buffer: &CommandBuffer, vertex_buffer: &Buffer, index_buffer: &Buffer, render_pass: &RenderPass, subpass_index: u32) {
        unsafe{
            device.get_ash_device().cmd_next_subpass(command_buffer.get_command_buffer(), vk::SubpassContents::INLINE);

            device.get_ash_device().cmd_bind_pipeline(command_buffer.get_command_buffer(),
             vk::PipelineBindPoint::GRAPHICS, render_pass.get_pipeline(subpass_index as usize));

            if self.instance_count == 0 {
                return;
            }

            device.get_ash_device().cmd_bind_vertex_buffers(
                command_buffer.get_command_buffer(), 0,
                 &[vertex_buffer.get_buffer(), self.instance_buffer.as_ref().unwrap().get_buffer()], &[0, 0]);
            device.get_ash_device().cmd_bind_index_buffer(command_buffer.get_command_buffer(), index_buffer.get_buffer(), 0, vk::IndexType::UINT16);    

            device.get_ash_device().cmd_draw_indexed(command_buffer.get_command_buffer(), 6, self.instance_count,
             0, 0, 0);
        }
    }

    pub fn get_required_vertex_input_states() -> ([vk::PipelineVertexInputStateCreateInfo<'a>; 1], VertexInputData){
        let vertex_bindings = vec![
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 8,
                input_rate: vk::VertexInputRate::VERTEX,
            },

            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 32,
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

                format: vk::Format::R32G32_UINT,
                offset: 0,
            },

            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 1,

                format: vk::Format::R32G32_UINT,
                offset: 8,
            },

            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 1,

                format: vk::Format::R32G32B32_UINT,
                offset: 16,
            },
 
        ];

        ([
            vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,

                vertex_attribute_description_count: vertex_attributes.len() as u32,
                p_vertex_attribute_descriptions: vertex_attributes.as_ptr(),

                vertex_binding_description_count: vertex_bindings.len() as u32,
                p_vertex_binding_descriptions: vertex_bindings.as_ptr(),

                ..Default::default()
            },
        ], (vertex_bindings, vertex_attributes))
    }

    pub fn create_buttons(&mut self, device: &Device, button_infos: &[(&Rect, Color, &str)], text_renderer: &TextRenderer) -> Vec<Button>{
        let mut buttons = Vec::<Button>::with_capacity(button_infos.len());

        self.creation_command_buffer.begin(device, &vk::CommandBufferInheritanceInfo::default(), vk::CommandBufferUsageFlags::empty());

        for info in button_infos {
            buttons.push(
                Button::new(device, &mut self.creation_command_buffer, text_renderer, *info.0, info.1, info.2)
            );
        }

        self.creation_command_buffer.end(device);

        CommandBuffer::submit(device, &[self.creation_command_buffer.get_command_buffer()], &[], &[], self.fence.get_fence());
         
        unsafe{
            device.get_ash_device().wait_for_fences(&[self.fence.get_fence()], true, std::u64::MAX)
             .expect("Failed to wait for the button creation fence.");

            device.get_ash_device().reset_fences(&[self.fence.get_fence()])
             .expect("Failed to reset the button manager creation fence");
        }

        self.creation_command_buffer.cleanup(device);

        buttons
    }

    pub fn destroy(&mut self, device: &Device) {
        match &mut self.instance_buffer {
            Some(buff) => buff.destroy(device),
            None => ()
        }
    }
}