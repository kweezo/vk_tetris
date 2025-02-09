use super::*;
use crate::{types::Rect, *};
use ash::vk;

pub struct TextManager {
    text_renderer:  TextRenderer,

    update_command_buffer: CommandBuffer,
    creation_command_buffer: CommandBuffer
}

impl TextManager {
    pub fn new(core: &Core, device: &Device, command_pool: &CommandPool) -> TextManager{
        let creation_command_buffer = CommandBuffer::new(device, command_pool, false);
        let update_command_buffer = CommandBuffer::new(device, command_pool, false);
        let text_renderer = TextRenderer::new(core, device, command_pool);

        TextManager { text_renderer, update_command_buffer, creation_command_buffer }
    }

    pub fn create_texts(&mut self, device: &Device, text_infos: &[(&str, &Rect)] ) -> Vec<Text> {
        self.creation_command_buffer.begin(device, &vk::CommandBufferInheritanceInfo::default(), vk::CommandBufferUsageFlags::empty());


        let mut texts = Vec::<Text>::with_capacity(text_infos.len());

        for info in text_infos.iter() {
            texts.push(
                Text::new(device, &mut self.creation_command_buffer, &self.text_renderer, info.0, *info.1)
            );  
        }

        self.creation_command_buffer.end(device);


        let fence = Fence::new(device, false);

        CommandBuffer::submit(device, &[self.creation_command_buffer.get_command_buffer()], &[], &[], fence.get_fence());

        unsafe {
            device.get_ash_device().wait_for_fences(&[fence.get_fence()], true, std::u64::MAX)
             .expect("Failed to wait for text create fence");
        }

        self.creation_command_buffer.cleanup(device);

        texts
    }

    pub fn change_texts(&mut self, device: &Device, texts: &mut [(&mut Text, &str)]) {
        self.update_command_buffer.begin(device, &vk::CommandBufferInheritanceInfo::default(), vk::CommandBufferUsageFlags::empty());

        for text in texts.iter_mut() {
            text.0.set_text(device, &self.text_renderer, &mut self.update_command_buffer, text.1);
        }

        self.update_command_buffer.end(device);


        let fence = Fence::new(device, false);


        CommandBuffer::submit(device, &[self.update_command_buffer.get_command_buffer()], &[], &[], fence.get_fence());

        unsafe {
            device.get_ash_device().wait_for_fences(&[fence.get_fence()], true, std::u64::MAX)
             .expect("Failed to wait for text update fence");
        }

        self.update_command_buffer.cleanup(device);

    }

    pub fn get_text_renderer(&self) -> &TextRenderer {
        &self.text_renderer
    }

    pub fn destroy(&mut self, device: &Device) {
        self.text_renderer.destroy(device);
    }
}