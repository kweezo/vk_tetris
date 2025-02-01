use ash::vk;

pub const PLAYFIELD_WIDTH: usize = 10;
pub const PLAYFIELD_HEIGHT: usize = 16;
pub type Grid = [[[u8; 4]; PLAYFIELD_WIDTH as usize]; PLAYFIELD_HEIGHT as usize];
pub type VertexInputData = (Vec<vk::VertexInputBindingDescription>, Vec<vk::VertexInputAttributeDescription>);
pub type Color = (u8, u8, u8); // todo replace all instances

#[derive(Clone, Copy, Debug)]
pub struct Rect{
    pub x: u32,
    pub y: u32,

    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn to_ne_bytes(&self) -> Vec<u8> {
        [
            self.x.to_ne_bytes(),
            self.y.to_ne_bytes(),

            self.width.to_ne_bytes(),
            self.height.to_ne_bytes()
        ].concat()
    }
}