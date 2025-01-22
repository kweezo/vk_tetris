use super::*;
use crate::*;

pub struct Text {
    text_buffer: Buffer,
    text_len: usize,
    scale: f32,
    pos: (u32, u32)
}

impl Text {
    pub fn new(device: &Device, command_buffer: &mut CommandBuffer, text_renderer: &TextRenderer, text_raw: &str, scale: f32, pos: (u32, u32)) -> Text{

        let text_buffer = Text::upload_text_to_buffer(device, command_buffer, text_renderer, text_raw);

        Text {pos, scale, text_len: text_raw.len(), text_buffer}
    }

    fn upload_text_to_buffer(device: &Device, command_buffer: &mut CommandBuffer, text_renderer: &TextRenderer, text_raw: &str, ) -> Buffer {

        let data = text_renderer.get_data_for_str(text_raw);

        let buffer = Buffer::new(device, command_buffer, &data, BufferType::Vertex, false);

        buffer
    }

    pub fn draw(&self, device: &Device, command_buffer: &CommandBuffer, text_renderer: &TextRenderer, render_pass: &RenderPass) {
        text_renderer.render_text(device, command_buffer, render_pass, &self.text_buffer, text_renderer::RenderInfo {
             char_count: self.text_len as u32, scale: self.scale,  pos: self.pos });
    }

    pub fn set_text(&mut self, device: &Device, text_renderer: &TextRenderer, command_buffer: &mut CommandBuffer, text_raw: &str) {
        
        if self.text_len != text_raw.len() {
            self.text_len = text_raw.len();

            self.text_buffer.destroy(device);
            self.text_buffer = Text::upload_text_to_buffer(device, command_buffer, text_renderer, text_raw);

            return
        }
        
        self.text_len = text_raw.len();

        let data = text_renderer.get_data_for_str(text_raw);

        self.text_buffer.update(device, command_buffer, &data);
    }

    pub fn set_pos(&mut self, new_pos: (u32, u32)) {
        self.pos = new_pos;
    }

    pub fn set_scale(&mut self, new_scale: f32) {
        self.scale = new_scale;
    }

    pub fn destroy(&mut self, device: &Device) {
        self.text_buffer.destroy(device);
    }
}