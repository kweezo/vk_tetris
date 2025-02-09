use types::{Color, Rect};

use crate::*;
use super::super::text::*;

pub struct Button {
    rect: Rect,
    color: Color,
    text: Text,
}

impl Button {
    pub fn new(device: &Device, command_buffer: &mut CommandBuffer, text_renderer: &TextRenderer, rect: Rect, color: Color, label: &str) -> Button{
        let text_padding = 0.4f32;
        //use tetromino tex for button and lighten up when hovered on

        let text_rect = Rect {
            x: rect.x + (rect.width as f32 * (text_padding / 2.0)) as u32,
            y: rect.y + (rect.height as f32 * (text_padding / 2.0)) as u32,
            width: (rect.width as f32 * (1.0 - text_padding)) as u32,
            height: (rect.height as f32* (1.0 - text_padding)) as u32,
        };

        let text = Text::new(device, command_buffer, text_renderer, label, text_rect);

        Button { rect: rect, text, color }
    }

    pub fn draw_text(&self, device: &Device, text_renderer: &TextRenderer, command_buffer: &CommandBuffer, render_pass: &RenderPass) {
        self.text.draw(device, command_buffer, text_renderer, render_pass);
    }

    pub fn get_raw_data(&self) -> Vec<u8> {
        /*
        vec2
        vec2
        vec3
        4 byte padding
         */
        [
            self.rect.x.to_ne_bytes(),
            self.rect.y.to_ne_bytes(),
            self.rect.width.to_ne_bytes(),
            self.rect.height.to_ne_bytes(),
            (self.color.0 as u32).to_ne_bytes(),
            (self.color.1 as u32).to_ne_bytes(),
            (self.color.2 as u32).to_ne_bytes(),
            1u32.to_ne_bytes()
        ].concat()
    }

    pub fn is_on_cursor(&self, mouse_pos: (u32, u32)) -> bool {
        if mouse_pos.0 > self.rect.x && mouse_pos.0 < self.rect.x + self.rect.width &&
           mouse_pos.1 > self.rect.y && mouse_pos.1 < self.rect.y + self.rect.height {
            return true;
        }

        false
    }

    pub fn get_name(&self) -> &String {
        self.text.get_text()
    }

    pub fn destroy(&mut self, device: &Device) {
        self.text.destroy(device);
    }
    
}