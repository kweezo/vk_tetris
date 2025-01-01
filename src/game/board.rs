use ash::vk;
use descriptor::{DescriptorInfo, DescriptorSet};
use std::time::{SystemTime, UNIX_EPOCH};
use vk_mem::Alloc;

use std::{
    mem,
    pin::Pin,
    sync::{Arc, RwLock},
};

use super::*;
use crate::vulkan::core::*;
use crate::vulkan::*;

use bytemuck::bytes_of;

const PLAYFIELD_WIDTH: usize = 10;
const PLAYFIELD_HEIGHT: usize = 16;

fn transfer_cleanup(
    device: ash::Device,
    allocator: Arc<RwLock<Arc<vk_mem::Allocator>>>,
    fence: vk::Fence,
    command_buffer: Arc<RwLock<CommandBuffer>>,
) {
    unsafe {
        device
            .wait_for_fences(&[fence], true, u64::MAX)
            .expect("Failed to wait for transfer fence");
        device
            .reset_fences(&[fence])
            .expect("Failed to reset the transfer fence");
    }

    command_buffer
        .write()
        .unwrap()
        .cleanup_raw(allocator.read().unwrap().allocator());
}

pub struct Board {
    tetromino: Tetromino,
    playfield: [[[u8; 3]; PLAYFIELD_HEIGHT]; PLAYFIELD_WIDTH],

    previous_tetromino_count: usize,

    tetromino_tex: Texture,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: Option<Buffer>,
    projection_uniform: Buffer,

    transfer_command_buffer: Arc<RwLock<CommandBuffer>>,

    transfer_finished_fence: Fence,

    transfer_finished_thread_handle: Option<std::thread::JoinHandle<()>>,

    transfer_semaphore: Semaphore,

    fall_interval: u32,
    previous_interval: u128,
}

impl<'a> Board {
    pub fn new(
        device: &Device,
        tetromino_tex_path: &str,
        command_pool: &CommandPool,
        screen_res: (u32, u32),
    ) -> Board {
        let mut transfer_command_buffer = CommandBuffer::new(device, command_pool, false);

        let transfer_semaphore = Semaphore::new(device);

        let buffers = Board::initialize_buffers(
            device,
            &mut transfer_command_buffer,
            tetromino_tex_path,
            screen_res,
        );

        Board {
            tetromino: Tetromino::new(4, 0, TetrominoType::I),
            transfer_command_buffer: Arc::new(RwLock::new(transfer_command_buffer)),
            instance_buffer: None,
            previous_tetromino_count: 0,
            transfer_semaphore,
            vertex_buffer: buffers.0,
            index_buffer: buffers.1,
            projection_uniform: buffers.2,
            tetromino_tex: buffers.3,
            transfer_finished_fence: Fence::new(device, false),
            transfer_finished_thread_handle: None,
            playfield: [[[0; 3]; 16]; 10],
            fall_interval: 250,
            previous_interval: 0,
        }
    }

    fn get_projection_matrix(screen_res: (u32, u32)) -> [f32; 16] {
        let left = 0f32;
        let right = screen_res.0 as f32;

        let bottom = 0f32;
        let top = screen_res.1 as f32;

        let near = -1f32;
        let far = 1f32;

        [
            2f32 / (right - left),
            0f32,
            0f32,
            -(right + left) / (right - left),
            0f32,
            2f32 / (top - bottom),
            0f32,
            -(top + bottom) / (top - bottom),
            0f32,
            0f32,
            -2f32 / (far - near),
            -(far + near) / (far - near),
            0f32,
            0f32,
            0f32,
            1f32,
        ]
    }

    fn initialize_buffers(
        device: &Device,
        command_buffer: &mut CommandBuffer,
        tetromino_tex_path: &str,
        screen_res: (u32, u32),
    ) -> (Buffer, Buffer, Buffer, Texture) {
        let indices: [u16; 6] = [0, 1, 2, 1, 2, 3];

        let vertices: [f32; 8] = [0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32];

        let fence = Fence::new(device, false);

        command_buffer.begin(
            device,
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        let vertex_buffer = Buffer::new(
            device,
            command_buffer,
            bytes_of(&vertices),
            BufferType::Vertex,
            false,
        );

        let index_buffer = Buffer::new(
            device,
            command_buffer,
            bytes_of(&indices),
            BufferType::Index,
            false,
        );

        let tetromino_tex = Texture::new(tetromino_tex_path, device, command_buffer)
            .expect("Failed to load the base tetromino texture");

        let projection = Board::get_projection_matrix(screen_res);
        let projection_buffer = Buffer::new(
            device,
            command_buffer,
            bytes_of(&projection),
            BufferType::Uniform,
            false,
        );

        command_buffer.end(device);

        CommandBuffer::submit(
            device,
            std::slice::from_ref(&command_buffer.get_command_buffer()),
            &[],
            &[],
            fence.get_fence(),
        );

        unsafe {
            device
                .get_ash_device()
                .wait_for_fences(&[fence.get_fence()], true, u64::MAX)
                .expect("Failed to wait for the board transfer fence");
        }

        command_buffer.cleanup(device);

        (
            vertex_buffer,
            index_buffer,
            projection_buffer,
            tetromino_tex,
        )
    }

    fn record_draw_command_buffer(
        &self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
        push_constants: &[u8],
        instance_count: u32,
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
                push_constants,
            );

            device.get_ash_device().cmd_bind_vertex_buffers(
                command_buffer.get_command_buffer(),
                0,
                &[
                    self.vertex_buffer.get_buffer(),
                    self.instance_buffer.as_ref().unwrap().get_buffer(),
                ],
                &[0, 0],
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
                instance_count,
                0,
                0,
                0,
            );
        }
    }

    fn get_instance_data(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::with_capacity(PLAYFIELD_HEIGHT * PLAYFIELD_WIDTH * 5);

        for x in 0..self.playfield.len() {
            for y in 0..self.playfield[x].len() {
                if self.playfield[x][y] == [0; 3] {
                    continue;
                }

                data.extend_from_slice(&[
                    x as u8,
                    y as u8,
                    self.playfield[x][y][0],
                    self.playfield[x][y][1],
                    self.playfield[x][y][2],
                ]);

                data.extend_from_slice(&[0; 3]); // padding
            }
        }

        for pos in self.tetromino.get_blocks().chunks_exact(2) {
            data.extend_from_slice(&[pos[0], pos[1], 255, 255, 255]);

            data.extend_from_slice(&[0; 3]);
        }

        data.shrink_to_fit();

        data
    }

    pub fn record_transfer_command_buffer(&mut self, device: &Device, data: &Vec<u8>) {
        self.transfer_command_buffer.read().unwrap().begin(
            device,
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        if data.len() != self.previous_tetromino_count {
            self.previous_tetromino_count = data.len();

            if let Some(buff) = &mut self.instance_buffer {
                buff.destroy(device)
            }

            self.instance_buffer = Some(Buffer::new(
                device,
                &mut self.transfer_command_buffer.write().unwrap(),
                data.as_slice(),
                BufferType::Vertex,
                false,
            ));
        } else {
            self.instance_buffer.as_mut().unwrap().update(
                device,
                &mut self.transfer_command_buffer.write().unwrap(),
                data.as_slice(),
            );
        }

        self.transfer_command_buffer.read().unwrap().end(device);
    }

    fn handle_inputs(&mut self, events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>) {
        for event in glfw::flush_messages(&events) {
            match event.1 {
                glfw::WindowEvent::Key(glfw::Key::R, _, glfw::Action::Press, _) => {
                    self.tetromino.rotate()
                }

                glfw::WindowEvent::Key(glfw::Key::Left, _, glfw::Action::Press, _) => {
                    self.tetromino.shift(-1, 0)
                }

                glfw::WindowEvent::Key(glfw::Key::Right, _, glfw::Action::Press, _) => {
                    self.tetromino.shift(1, 0)
                }

                _ => (),
            }
        }
    }

    fn handle_gravity(&mut self) {
        let curr = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        if (self.previous_interval + self.fall_interval as u128) >= curr {
            return;
        }

        self.previous_interval = curr;

        self.tetromino.shift(0, 1);
    }

    pub fn update(&mut self, events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>) {
        self.handle_inputs(events);
        self.handle_gravity();
    }

    pub fn draw(
        &mut self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
    ) {
        let data = Board::get_instance_data(self);

        if data.is_empty() {
            return;
        }
        self.record_transfer_command_buffer(device, &data);
        self.record_draw_command_buffer(
            device,
            render_pass,
            command_buffer,
            subpass_index,
            &[0; 128],
            (data.len() / 8) as u32,
        );

        CommandBuffer::submit(
            device,
            &[self
                .transfer_command_buffer
                .read()
                .unwrap()
                .get_command_buffer()],
            &[],
            &[self.transfer_semaphore.get_semaphore()],
            self.transfer_finished_fence.get_fence(),
        );

        match mem::take(&mut self.transfer_finished_thread_handle) {
            None => (),
            Some(handle) => handle.join().unwrap(),
        };

        self.transfer_finished_thread_handle = Some(std::thread::spawn({
            let allocator = device.get_allocator_lock();
            let device = device.get_ash_device().clone();
            let fence = self.transfer_finished_fence.get_fence();
            let command_buffer = self.transfer_command_buffer.clone();

            move || {
                transfer_cleanup(device, allocator, fence, command_buffer);
            }
        }));
    }

    pub fn add_piece(&mut self, x: u8, y: u8, shape: tetromino::TetrominoType) {
        self.tetromino = Tetromino::new(x, y, shape);
    }

    pub fn get_tetromino_tex(&self) -> &Texture {
        &self.tetromino_tex
    }

    pub fn get_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet; 2], Pin<Box<[DescriptorInfo; 2]>>) {
        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.tetromino_tex.get_image_view(),
            sampler: self.tetromino_tex.get_sampler(),
        };

        let buffer_info = vk::DescriptorBufferInfo {
            buffer: self.projection_uniform.get_buffer(),
            offset: 0,
            range: vk::WHOLE_SIZE,
        };

        let descriptor_infos = Pin::new(Box::new([
            descriptor::DescriptorInfo::Image(vec![image_info]),
            descriptor::DescriptorInfo::Buffer(vec![buffer_info]),
        ]));

        let image_write_set = set.create_write_set(
            &descriptor_infos.as_ref()[0],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            0,
            1,
        );
        let uniform_write_set = set.create_write_set(
            &descriptor_infos.as_ref()[1],
            vk::DescriptorType::UNIFORM_BUFFER,
            0,
            1,
        );

        ([image_write_set, uniform_write_set], descriptor_infos)
    }

    pub fn get_transfer_semaphore(&self) -> vk::Semaphore {
        self.transfer_semaphore.get_semaphore()
    }

    pub fn destruct(&mut self, device: &Device) {
        self.tetromino_tex.destroy(device);
        self.index_buffer.destroy(device);
        self.vertex_buffer.destroy(device);
        self.instance_buffer.as_mut().unwrap().destroy(device);
        self.projection_uniform.destroy(device);
    }
}
