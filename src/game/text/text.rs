use super::*;
use crate::{types::Rect, *};

pub struct Text {
    text_buffer: Buffer,
    text_len: usize,
    rect: Rect,
    text_raw: String

}

impl Text {
    pub fn new(device: &Device, command_buffer: &mut CommandBuffer, text_renderer: &TextRenderer, text_raw: &str, rect: Rect) -> Text{

        let (text_buffer, size, y_offset) = Text::upload_text_to_buffer(device, command_buffer, text_renderer, text_raw, &rect);

        Text {rect: Rect { x: rect.x, y: rect.y - y_offset, width: size.0, height: size.1 }, text_len: text_raw.len(), text_buffer, text_raw: String::from(text_raw)}
    }

    fn upload_text_to_buffer(device: &Device, command_buffer: &mut CommandBuffer, text_renderer: &TextRenderer, text_raw: &str, rect: &Rect) -> (Buffer, (u32, u32), u32){

        let (data, size, y_offset) = text_renderer.get_data_for_str(text_raw, rect);

        let buffer = Buffer::new(device, command_buffer, &data, BufferType::Vertex, false);

        (buffer, size, y_offset)
    }

    pub fn draw(&self, device: &Device, command_buffer: &CommandBuffer, text_renderer: &TextRenderer, render_pass: &RenderPass) {
        text_renderer.render_text(device, command_buffer, render_pass, &self.text_buffer, text_renderer::RenderInfo {
             char_count: self.text_len as u32, rect: self.rect });
    }

    pub fn set_text(&mut self, device: &Device, text_renderer: &TextRenderer, command_buffer: &mut CommandBuffer, text_raw: &str) {
        
        if self.text_len != text_raw.len() {
            self.text_len = text_raw.len();

            self.text_buffer.destroy(device);
            (self.text_buffer, (self.rect.width, self.rect.height), _) = Text::upload_text_to_buffer(device, command_buffer, text_renderer, text_raw, &self.rect);

            return
        }
        
        self.text_len = text_raw.len();

        let (data, _, _) = text_renderer.get_data_for_str(text_raw, &self.rect);

        self.text_buffer.update(device, command_buffer, &data);
    }

    pub fn set_pos(&mut self, new_pos: (u32, u32)) {
        self.rect.x = new_pos.0;
        self.rect.y = new_pos.1;
    }

    pub fn set_size(&mut self, new_size: (u32, u32)) {
        self.rect.width = new_size.0;
        self.rect.width = new_size.1;
    }

    pub fn get_text(&self) -> &String{
        &self.text_raw
    }

    pub fn destroy(&mut self, device: &Device) {
        self.text_buffer.destroy(device);
    }
}