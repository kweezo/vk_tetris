use ash::vk;
use descriptor::{DescriptorInfo, DescriptorSet};
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
    tetrominos: Vec<Tetromino>,
    previous_tetromino_count: usize,

    base_tetromino_tex: Texture,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: Option<Buffer>,
    projection_uniform: Buffer,

    transfer_command_buffer: Arc<RwLock<CommandBuffer>>,
    draw_command_buffer: CommandBuffer,

    transfer_finished_fence: Fence,

    transfer_finished_thread_handle: Option<std::thread::JoinHandle<()>>,

    transfer_semaphore: Semaphore,
    draw_semaphore: Semaphore,
}

impl<'a> Board {
    pub fn new(
        device: &Device,
        base_tetromino_tex_path: &str,
        command_pool: &CommandPool,
        screen_res: (u32, u32),
    ) -> Board {
        let indices: [u16; 6] = [0, 1, 2, 1, 2, 3];

        let vertices: [f32; 8] = [0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32];

        let mut transfer_command_buffer = CommandBuffer::new(device, command_pool, false);
        let draw_command_buffer = CommandBuffer::new(device, command_pool, true);

        let fence = Fence::new(device, false);

        transfer_command_buffer.begin(
            device,
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        let vertex_buffer = Buffer::new(
            device,
            &mut transfer_command_buffer,
            bytes_of(&vertices),
            BufferType::Vertex,
            false,
        );
        let index_buffer = Buffer::new(
            device,
            &mut transfer_command_buffer,
            bytes_of(&indices),
            BufferType::Index,
            false,
        );
        let base_tetromino_tex = Texture::new(
            base_tetromino_tex_path,
            device,
            &mut transfer_command_buffer,
        )
        .expect("Failed to load the base tetromino texture");

        let projection = Board::get_projection_matrix(screen_res);
        let projection_uniform = Buffer::new(
            device,
            &mut transfer_command_buffer,
            bytes_of(&projection),
            BufferType::Uniform,
            false,
        );

        transfer_command_buffer.end(device);

        CommandBuffer::submit(
            device,
            std::slice::from_ref(&transfer_command_buffer.get_command_buffer()),
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

        transfer_command_buffer.cleanup(device);

        let transfer_semaphore = Semaphore::new(device);
        let draw_semaphore = Semaphore::new(device);

        Board {
            tetrominos: Vec::new(),
            base_tetromino_tex,
            transfer_command_buffer: Arc::new(RwLock::new(transfer_command_buffer)),
            draw_command_buffer,
            vertex_buffer,
            index_buffer,
            instance_buffer: None,
            previous_tetromino_count: 0,
            transfer_semaphore,
            draw_semaphore,
            projection_uniform,
            transfer_finished_fence: Fence::new(device, false),
            transfer_finished_thread_handle: None,
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

    fn record_draw_command_buffer(
        &self,
        device: &Device,
        render_pass: &RenderPass,
        subpass_index: u32,
        image_index: u32,
        set: &DescriptorSet,
        push_constants: &[u8],
    ) {
        let inheritance_info = vk::CommandBufferInheritanceInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_INHERITANCE_INFO,

            framebuffer: render_pass.get_framebuffer(image_index),
            render_pass: render_pass.get_render_pass(),
            subpass: subpass_index,

            ..Default::default()
        };

        self.draw_command_buffer.begin(
            device,
            &inheritance_info,
            vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
        );

        let offset = 0u32;

        unsafe {
            device.get_ash_device().cmd_bind_pipeline(
                self.draw_command_buffer.get_command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                render_pass.get_pipeline(),
            );

            device.get_ash_device().cmd_bind_descriptor_sets(
                self.draw_command_buffer.get_command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                render_pass.get_layout(),
                0,
                &[set.get_set()],
                &[offset],
            );

            device.get_ash_device().cmd_push_constants(
                self.draw_command_buffer.get_command_buffer(),
                render_pass.get_layout(),
                vk::ShaderStageFlags::ALL,
                offset,
                push_constants,
            );

            device.get_ash_device().cmd_bind_vertex_buffers(
                self.draw_command_buffer.get_command_buffer(),
                0,
                &[
                    self.vertex_buffer.get_buffer(),
                    self.instance_buffer.as_ref().unwrap().get_buffer(),
                ],
                &[0, 0],
            );
            device.get_ash_device().cmd_bind_index_buffer(
                self.draw_command_buffer.get_command_buffer(),
                self.index_buffer.get_buffer(),
                0,
                vk::IndexType::UINT16,
            );
            device.get_ash_device().cmd_draw_indexed(
                self.draw_command_buffer.get_command_buffer(),
                6,
                self.tetrominos.len() as u32 * 4,
                0,
                0,
                0,
            );
        }

        self.draw_command_buffer.end(device);
    }

    fn get_instance_data(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::with_capacity(self.tetrominos.len() * 8);

        for tetromino in &self.tetrominos {
            let block_positions = tetromino.get_blocks();
            for pos in block_positions.chunks(2) {
                data.extend_from_slice(pos); // pos
                data.extend_from_slice(&[255; 3]); // color
                data.extend_from_slice(&[1; 3]); // padding
            }
        }

        data
    }

    pub fn record_transfer_command_buffer(&mut self, device: &Device) {
        self.transfer_command_buffer.read().unwrap().begin(
            device,
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        let data = Board::get_instance_data(self);

        if self.tetrominos.len() != self.previous_tetromino_count {
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

    pub fn update() {}

    pub fn draw(
        &mut self,
        device: &Device,
        render_pass: &RenderPass,
        subpass_index: u32,
        image_index: u32,
        set: &DescriptorSet,
        push_constants: &[u8],
    ) -> Option<(vk::CommandBuffer, vk::Semaphore)> {
        if self.tetrominos.is_empty() {
            return None;
        }

        self.record_transfer_command_buffer(device);
        self.record_draw_command_buffer(
            device,
            render_pass,
            subpass_index,
            image_index,
            set,
            push_constants,
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

        Some((
            self.draw_command_buffer.get_command_buffer(),
            self.transfer_semaphore.get_semaphore(),
        ))
    }

    pub fn add_piece(&mut self, x: u8, y: u8, shape: tetromino::TetrominoType) {
        self.tetrominos.push(Tetromino::new(x, y, shape));
    }

    pub fn get_tetromino_tex(&self) -> &Texture {
        &self.base_tetromino_tex
    }

    pub fn get_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet; 2], Pin<Box<[DescriptorInfo; 2]>>) {
        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.base_tetromino_tex.get_image_view(),
            sampler: self.base_tetromino_tex.get_sampler(),
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
            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            0,
            1,
        );

        ([image_write_set, uniform_write_set], descriptor_infos)
    }

    pub fn destruct(&mut self, device: &Device) {
        self.base_tetromino_tex.destroy(device);
        self.index_buffer.destroy(device);
        self.vertex_buffer.destroy(device);
        self.instance_buffer.as_mut().unwrap().destroy(device);
        self.projection_uniform.destroy(device);
    }
}
