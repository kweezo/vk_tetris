use ash::vk;

pub const PLAYFIELD_WIDTH: usize = 10;
pub const PLAYFIELD_HEIGHT: usize = 16;
pub type Grid = [[[u8; 4]; PLAYFIELD_WIDTH as usize]; PLAYFIELD_HEIGHT as usize];
pub type VertexInputData = Vec<(Vec<vk::VertexInputBindingDescription>, Vec<vk::VertexInputAttributeDescription>)>;