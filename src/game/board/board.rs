use ash::vk;
use descriptor::{DescriptorInfo, DescriptorSet};
use std::{sync::Mutex, time::{SystemTime, UNIX_EPOCH}};
use super::super::*;

use std::{
    pin::Pin,
    sync::Arc,
};

use super::*;
use crate::{vulkan::{core::*, *}, types::*};

use bytemuck::bytes_of;

use rand::prelude::*;


#[derive(Clone, Copy)]
pub enum GameState{
    RUNNING,
    END
}

#[inline]
fn lerp(x: f32, y: f32, a: f32) -> f32{
    x * (1.0 - a) + y * a
}

#[inline]
fn length(x: f32, y: f32) -> f32{
    (x.powf(2.0) + y.powf(2.0)).sqrt()
}

struct ScreenShake {
    points: [(f32, f32); 12],
    last_dir_change: u64,
    curr_point_index: usize,
}

impl ScreenShake {
    pub fn new() -> ScreenShake {
        let mut rng = rand::rng();

        let mut points = [(0f32, 0f32); 12];

        for p in points.iter_mut() {
            *p = (rng.random_range(-10..10) as f32,
            rng.random_range(-10..10) as f32);
        }

        ScreenShake { points, last_dir_change: 0, curr_point_index: 0 }
    }

    pub fn update(&mut self, curr_time: u64) -> (f32, f32) {

        if curr_time - self.last_dir_change > 20 {
            self.last_dir_change = curr_time;
            self.curr_point_index += 1;
        }

        if self.curr_point_index >= self.points.len() {
            return (std::f32::NAN, std::f32::NAN);
        }
        
        (self.points[self.curr_point_index].0, self.points[self.curr_point_index].1)
    }
}


pub struct Board {
    tetromino: Tetromino,
    grid: Grid,

    previous_tetromino_count: usize,

    tetromino_tex: Texture,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: Option<Buffer>,
    projection_uniform: Buffer,

    transfer_command_buffer: CommandBuffer,

    transfer_finished_fence: Fence,

    fall_interval: u32,
    previous_interval: u128,

    score: Arc<Mutex<u32>>,

    game_state: GameState,

    tetromino_instance_count: u32,

    rng: ThreadRng,

    place_sound: Sound,

    screen_shake: Option<ScreenShake>
}

impl<'a> Board {
    pub fn new(
        device: &Device,
        command_pool: &CommandPool,
    ) -> Board {
        let mut transfer_command_buffer = CommandBuffer::new(device, command_pool, false);

        let buffers = Board::initialize_buffers(
            device,
            &mut transfer_command_buffer,
            "tetromino_piece.png",
        );

        let place_sound = Sound::new("place.wav",
        2.0, false);

        let mut rng = rand::rng();

        let mut tetromino = Tetromino::new((3, 2), [255; 3], TetrominoShape::rand(&mut rng, TetrominoShape::I));
        tetromino.translate((0, 0), &[[[0; 4]; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT]);

        Board {
            tetromino,
            transfer_command_buffer: transfer_command_buffer,
            instance_buffer: None,
            previous_tetromino_count: 0,
            vertex_buffer: buffers.0,
            index_buffer: buffers.1,
            projection_uniform: buffers.2,
            tetromino_tex: buffers.3,
            transfer_finished_fence: Fence::new(device, false),
            grid: [[[0; 4]; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT],
            fall_interval: 1500,
            previous_interval: 0,
            tetromino_instance_count: 0,
            rng,
            game_state: GameState::RUNNING,
            score: Arc::new(Mutex::new(0)),
            place_sound,
            screen_shake: None
        }
    }

    fn get_projection_matrix(offset: (f32, f32)) -> [f32; 16] {
        let screen_res = (720, 1280);

        let left = 0f32 + offset.0;
        let right = screen_res.0 as f32 + offset.0;

        let bottom = 0f32 + offset.1;
        let top = screen_res.1 as f32 + offset.1;

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

        let tetromino_tex = Texture::new(tetromino_tex_path, device, command_buffer, false)
            .expect("Failed to load the base tetromino texture");

        let projection = Board::get_projection_matrix((0.0, 0.0));
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
    ) {
        let offset = 0u32;

        let push_constants = [[0u8; 4], self.tetromino_instance_count.to_ne_bytes()].concat();

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
                &push_constants,
            );

            device.get_ash_device().cmd_bind_vertex_buffers(
                command_buffer.get_command_buffer(),
                0,
                &[
                    self.vertex_buffer.get_buffer(),
                ],
                &[0],
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
                self.tetromino_instance_count,
                0,
                0,
                0,
            );
        }
    }

    fn get_instance_data(&mut self) -> Vec<u8> {
        let mut data = Vec::<u8>::with_capacity(PLAYFIELD_HEIGHT * PLAYFIELD_WIDTH * 20);

        for y in 0..self.grid.len() {
            for x in 0..self.grid[y].len() {
                if self.grid[y][x] == [0; 4] {
                    continue;
                }

                data.extend_from_slice(&[
                    (self.grid[y][x][0] as u32).to_ne_bytes(),
                    (self.grid[y][x][1] as u32).to_ne_bytes(),
                    (self.grid[y][x][2] as u32).to_ne_bytes(),
                    (self.grid[y][x][3] as u32).to_ne_bytes(),
                    (x as u32).to_ne_bytes(),
                    (y as u32).to_ne_bytes(),
                    0u32.to_ne_bytes(),
                    0u32.to_ne_bytes(),
                ].concat());
            }
        }

        for pos in self.tetromino.get_data().chunks(2) {
            let color = self.tetromino.get_color();

            data.extend_from_slice(&[
                (color[0] as u32).to_ne_bytes(),
                (color[1] as u32).to_ne_bytes(),
                (color[2] as u32).to_ne_bytes(),
                (color[3] as u32).to_ne_bytes(),
            ].concat());

            data.extend_from_slice(&[
                (pos[0] as u32).to_ne_bytes(),
                (pos[1] as u32).to_ne_bytes(),
            ].concat());

            data.extend_from_slice(&[0u8; 8]);
        }

        for pos in self.tetromino.get_ghost_data(&self.grid).chunks(2) {
            let color = self.tetromino.get_ghost_color();

            data.extend_from_slice(&[
                (color[0] as u32).to_ne_bytes(),
                (color[1] as u32).to_ne_bytes(),
                (color[2] as u32).to_ne_bytes(),
                (color[3] as u32).to_ne_bytes(),
            ].concat());


            data.extend_from_slice(&[
                (pos[0] as u32).to_ne_bytes(),
                (pos[1] as u32).to_ne_bytes(),
            ].concat());

            data.extend_from_slice(&[0u8; 8]);
        }

        data.extend_from_slice(&[0u8; 8]);

        data.shrink_to_fit();

        self.tetromino_instance_count = (data.len() / 32) as u32;

        data
    }

    fn handle_screen_shake(&mut self, device: &Device, time: u64) {
        if self.screen_shake.is_none() {
            return;
        }

        let mut pos = self.screen_shake.as_mut().unwrap().update(time);

        if pos.0.is_nan() || pos.1.is_nan() {
            self.screen_shake = None;
            pos = (0.0, 0.0);
        }


        let projection = Board::get_projection_matrix(pos);

        self.projection_uniform.update(device, &mut self.transfer_command_buffer, bytes_of(&projection));

    }

    pub fn handle_transfer(&mut self, device: &Device, data: &Vec<u8>, time: u64) {
        self.transfer_command_buffer.begin(
            device,
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        self.handle_screen_shake(device, time);

        if data.len() != self.previous_tetromino_count {
            self.previous_tetromino_count = data.len();

            if let Some(buff) = &mut self.instance_buffer {
                buff.destroy(device)
            }

            self.instance_buffer = Some(Buffer::new(
                device,
                &mut self.transfer_command_buffer,
                data.as_slice(),
                BufferType::Storage,
                true,
            ));
        } else {
            self.instance_buffer.as_mut().unwrap().update(
                device,
                &mut self.transfer_command_buffer,
                data.as_slice(),
            );
        }

        self.transfer_command_buffer.end(device);


        CommandBuffer::submit(device, &[self.transfer_command_buffer.get_command_buffer()], &[], &[], self.transfer_finished_fence.get_fence());

        unsafe {
            device
                .get_ash_device()
                .wait_for_fences(&[self.transfer_finished_fence.get_fence()], true, u64::MAX)
                .expect("Failed to wait for transfer fence");
            device
                .get_ash_device()
                .reset_fences(&[self.transfer_finished_fence.get_fence()])
                .expect("Failed to reset the transfer fence");
        }
        
        self.transfer_command_buffer.cleanup(device);
    }

    fn check_collision_in_dir(&self, x: i8, y: i8) -> bool {
        for pos in self.tetromino.get_data().chunks(2) {
            let offset_x = ((pos[0] as i8) + x) as usize;
            let offset_y = ((pos[1] as i8) + y) as usize;

            if offset_x >= PLAYFIELD_WIDTH || offset_y >= PLAYFIELD_HEIGHT {
                return false;
            }

            if self.grid[offset_y][offset_x] != [0; 4] {
                return true;
            }
        }

        false
    }

    fn get_random_color(&mut self) -> [u8; 3] {
        match self.rng.random_range(1..7) {
            1 => [255, 0, 0],
            2 => [0, 0, 255],
            3 => [0, 255, 0],
            4 => [255, 255, 0],
            5 => [255, 0, 255],
            6 => [0, 255, 255],
            _ => [255, 255, 255],
        }
    }

    fn handle_inputs(&mut self, events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>, audio_manager: &mut AudioManager) {
        for event in glfw::flush_messages(&events) {
            match event.1 {
                glfw::WindowEvent::Key(glfw::Key::R, _, glfw::Action::Press, _) => {
                    self.tetromino.rotate(tetromino::Orientation::RIGHT, &self.grid)
                }

                glfw::WindowEvent::Key(glfw::Key::Left, _, glfw::Action::Press, _) => {
                    self.tetromino.translate((-1, 0), &self.grid);
                }

                glfw::WindowEvent::Key(glfw::Key::Right, _, glfw::Action::Press, _) => {
                    self.tetromino.translate((1, 0), &self.grid);
                }

                glfw::WindowEvent::Key(glfw::Key::Down, _, glfw::Action::Press, _) => {
                    self.handle_block_collision(audio_manager);
                    self.tetromino.translate((0, 1), &self.grid);

                    self.previous_interval = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                }

                glfw::WindowEvent::Key(glfw::Key::Space, _, glfw::Action::Press, _) => {
                    while self.tetromino.translate((0, 1), &self.grid) {}
                    self.handle_block_collision(audio_manager);
                },

                glfw::WindowEvent::Key(glfw::Key::N, _, glfw::Action::Press, _) => {
                    self.reset_game();
                },

                _ => (),
            }
        }
    }

    fn handle_gravity(&mut self) {
        self.tetromino.translate((0, 1), &self.grid);
    }

    pub fn reset_game(&mut self) {
        self.grid = [[[0; 4]; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT];
        self.game_state = GameState::RUNNING;
        
        self.add_tetromino(0, 0);

    }

    fn handle_block_collision(&mut self, audio_manager: &mut AudioManager) {

        if self.tetromino.is_topped_out() {
            self.game_state = GameState::END;
            return;
        }

        if !self.tetromino.is_grounded(&self.grid) {
            self.game_state = GameState::RUNNING;
            return;
        }

        for pos in self.tetromino.get_data().chunks(2) {
            self.grid[pos[1] as usize][pos[0] as usize] = self.tetromino.get_color();
        }

        audio_manager.play(&mut self.place_sound);
        
        self.add_tetromino(2, 2);

        if !self.tetromino.is_valid(&self.grid) {
            self.game_state = GameState::END;
        }
    }

    fn handle_line_clear(&mut self) {
        let mut consecutive_clears = 0u8;

        for y in 0..PLAYFIELD_HEIGHT {
            let mut is_full = true;

            for x in 0..PLAYFIELD_WIDTH {
                if self.grid[y][x] == [0; 4] {
                    is_full = false;
                }
            }

            if !is_full {
                continue;
            }

            consecutive_clears += 1;

            
            for y_new in (0..y).rev() {
                self.grid[y_new + 1] = self.grid[y_new];
            }
        }

        if consecutive_clears != 0 {
            self.screen_shake = Some(ScreenShake::new());
        }


        *self.score.lock().expect("Failed to lock") +=  match consecutive_clears {
            0 => 0,
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 2500
        };
    }

    fn fixed_update(&mut self, audio_manager: &mut AudioManager) {
        let curr = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        if (self.previous_interval + self.fall_interval as u128) >= curr {
            return;
        }

        self.previous_interval = curr;

        self.handle_block_collision(audio_manager);
        self.handle_gravity();

    }

    pub fn update(&mut self, events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>, audio_manager: &mut AudioManager) {
        self.fixed_update(audio_manager);
        self.handle_inputs(events, audio_manager);
        self.handle_line_clear();
    }

    pub fn draw(
        &mut self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
        time: u64
    ) {
        let data = Board::get_instance_data(self);

        if data.is_empty() {
            return;
        }
        self.handle_transfer(device, &data, time);
        self.record_draw_command_buffer(
            device,
            render_pass,
            command_buffer,
            subpass_index,
        );
    }

    pub fn add_tetromino(&mut self, x: i8, y: i8) {
        self.tetromino = Tetromino::new(
            (x, y),
            self.get_random_color(),
            TetrominoShape::rand(&mut self.rng, self.tetromino.get_shape()),
        );

        let mut scalar = 0;


        while !self.tetromino.is_in_bounds() {
            self.tetromino.translate((scalar, scalar), &self.grid);
            scalar += 1;
        }

        while self.tetromino.translate((0, -1), &self.grid){}
  
    }

    pub fn get_tetromino_tex(&self) -> &Texture {
        &self.tetromino_tex
    }

    pub fn get_score(&self) -> Arc<Mutex<u32>> {
        self.score.clone()
    }

    pub fn get_tetromino_instance_count(&self) -> u32 {
        self.tetromino_instance_count
    }

    pub fn get_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet<'a>; 2], Pin<Box<[DescriptorInfo; 2]>>) {
        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.tetromino_tex.get_image_view(),
            sampler: self.tetromino_tex.get_sampler(),
        };

        let buffer_info_projection = vk::DescriptorBufferInfo {
            buffer: self.projection_uniform.get_buffer(),
            offset: 0,
            range: vk::WHOLE_SIZE,
        };


        let descriptor_infos = Pin::new(Box::new([
            descriptor::DescriptorInfo::Image(vec![image_info]),
            descriptor::DescriptorInfo::Buffer(vec![buffer_info_projection]),
        ]));

        let image_write_set = set.create_write_set(
            &descriptor_infos.as_ref()[0],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            0,
            1,
            1
        );

        let buffer_write_set_projection = set.create_write_set(
            &descriptor_infos.as_ref()[1],
            vk::DescriptorType::UNIFORM_BUFFER,
            0,
            1,
            6
        );


        ([image_write_set, buffer_write_set_projection], descriptor_infos)
    }

    pub fn instance_buffer_exists(&self) -> bool {
        match self.instance_buffer {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_instance_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet<'a>; 1], Pin<Box<[DescriptorInfo; 1]>>) {

        let buffer_info_instance = vk::DescriptorBufferInfo {
            buffer: self.instance_buffer.as_ref().unwrap().get_buffer(),
            offset: 0,
            range: vk::WHOLE_SIZE,
        };

        let descriptor_infos = Pin::new(Box::new([
            descriptor::DescriptorInfo::Buffer(vec![buffer_info_instance]),
        ]));

        let buffer_write_set_instance_dat = set.create_write_set(
            &descriptor_infos.as_ref()[0],
            vk::DescriptorType::STORAGE_BUFFER,
            0,
            1,
            8
        );

        ([buffer_write_set_instance_dat], descriptor_infos)
    }

    pub fn get_game_state(&self) -> GameState {
        self.game_state
    }

    pub fn get_required_vertex_input_states() -> ([vk::PipelineVertexInputStateCreateInfo<'a>; 1], VertexInputData){
        let vertex_bindings = vec![
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 8,
                input_rate: vk::VertexInputRate::VERTEX,
            },

        ];

        let vertex_attributes = vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,

                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
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
            }
        ],
        (vertex_bindings, vertex_attributes))

    }

    pub fn destruct(&mut self, device: &Device) {
        self.tetromino_tex.destroy(device);
        self.index_buffer.destroy(device);
        self.vertex_buffer.destroy(device);
        self.instance_buffer.as_mut().unwrap().destroy(device);
        self.projection_uniform.destroy(device);
    }
}
