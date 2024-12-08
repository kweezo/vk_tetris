use ash::vk;

use super::*;
use crate::vulkan::*;
use crate::vulkan::core::*;

use bytemuck::bytes_of;

pub struct Board{
    tetrominos: Vec<Tetromino>,
    previous_tetromino_count: usize,
    

    base_tetromino_tex: Texture,


    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: Option<Buffer>,

    transfer_command_buffer: CommandBuffer,
    draw_command_buffer: CommandBuffer,

    transfer_semaphore: Semaphore,
    draw_semaphore: Semaphore
}

impl Board{
    pub fn new(device: &Device, base_tetromino_tex_path: &str, command_pool: CommandPool) -> Board{

        let indices: [u16; 6] = [
            0, 1, 2,
            1, 2, 3
        ];

        let vertices: [f32; 8] = [
            -0.5, -0.5,
            -0.5,  0.5,
             0.5, -0.5,
             0.5,  0.5
        ];

        let mut transfer_command_buffer = CommandBuffer::new(device, &command_pool);
        let draw_command_buffer = CommandBuffer::new(device, &command_pool);

        let fence = Fence::new(device, false);


        transfer_command_buffer.begin(device);

        let vertex_buffer = Buffer::new(device, &mut transfer_command_buffer, bytes_of(&vertices), BufferType::VERTEX, false);
        let index_buffer = Buffer::new(device, &mut transfer_command_buffer, bytes_of(&indices), BufferType::INDEX, false);
        let base_tetromino_tex = Texture::new(base_tetromino_tex_path, device, &mut transfer_command_buffer).expect("Failed to load the base tetromino texture");

        transfer_command_buffer.end(device);


        CommandBuffer::submit(device, std::slice::from_ref(&&transfer_command_buffer), &[], &[], fence.get_fence());

        unsafe{
            device.get_ash_device().wait_for_fences(&[fence.get_fence()], true, std::u64::MAX).expect("Failed to wait for the board transfer fence");
        }

        let transfer_semaphore = Semaphore::new(device);
        let draw_semaphore = Semaphore::new(device);

        Board{tetrominos: Vec::new(), base_tetromino_tex: base_tetromino_tex, transfer_command_buffer, draw_command_buffer: draw_command_buffer, vertex_buffer: vertex_buffer, index_buffer: index_buffer, instance_buffer: None, previous_tetromino_count: 0, transfer_semaphore: transfer_semaphore, draw_semaphore: draw_semaphore}
    }

    fn record_command_buffer(&self, device: &Device){
        self.draw_command_buffer.begin(device);
        
        unsafe{
            device.get_ash_device().cmd_bind_vertex_buffers(self.draw_command_buffer.get_command_buffer(), 0, &[self.vertex_buffer.get_buffer(), self.instance_buffer.as_ref().unwrap().get_buffer()], &[0, 0]);

            device.get_ash_device().cmd_bind_index_buffer(self.draw_command_buffer.get_command_buffer(), self.index_buffer.get_buffer(), 0, vk::IndexType::UINT16);

            device.get_ash_device().cmd_draw_indexed(self.draw_command_buffer.get_command_buffer(), 6, self.tetrominos.len() as u32, 0, 0, 0);
        }

        self.draw_command_buffer.end(device);
    }

    fn get_instance_data(&self) -> Vec<u8>{
        let mut data = Vec::<u8>::with_capacity(self.tetrominos.len() * 8);

        for tetromino in &self.tetrominos{
            data.extend_from_slice(&tetromino.get_blocks()); // pos
            data.extend_from_slice(&[1; 3]); // color
            data.extend_from_slice(&[0; 3]); // padding
        }

        data
    }

    pub fn draw(&mut self, device: &Device) -> Option<vk::Semaphore>{
        if self.tetrominos.is_empty(){
            return None
        }


        let data = Board::get_instance_data(&self);


        if self.tetrominos.len() != self.previous_tetromino_count{
            self.instance_buffer = Some(Buffer::new(device, &mut self.transfer_command_buffer, data.as_slice(), BufferType::VERTEX, false));
        }else{
            self.instance_buffer.as_mut().unwrap().update(device, &mut self.transfer_command_buffer, data.as_slice());
        }

        self.record_command_buffer(device);


        CommandBuffer::submit(device, &[&self.transfer_command_buffer], &[], &[&self.transfer_semaphore], vk::Fence::null());
        CommandBuffer::submit(device, &[&self.draw_command_buffer], &[(&self.transfer_semaphore, vk::PipelineStageFlags::VERTEX_INPUT)], &[&self.draw_semaphore], vk::Fence::null());

        return Some(self.draw_semaphore.get_semaphore())
    }

    pub fn destruct(&mut self, device: &Device){
        self.base_tetromino_tex.destroy(device);
    }
}