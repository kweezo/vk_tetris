use crate::{game::board::Board, types::*, *};
use ash::vk;
use bytemuck::bytes_of;
use descriptor::{DescriptorInfo, DescriptorSet};
use std::{pin::Pin, sync::{Arc, Mutex}};
use super::super::{text::*, button::*};

use super::Backdrop;


pub struct UserInterface {

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    text_manager: TextManager,

    backdrop: Backdrop,

    score_text: Text,
    end_text: Text,

    button_manager: ButtonManager,
    reset_button: Button,

    last_pressed: bool,

    game_state: GameState,

    score: Arc<Mutex<u32>>
}

impl<'a> UserInterface {
    pub fn new(core: &Core, device: &Device, command_pool: &CommandPool, score: Arc<Mutex<u32>>) -> UserInterface {
        let mut text_manager = TextManager::new(core, device, command_pool);

        let buffers = UserInterface::initialize_buffers(device, command_pool);

        let mut texts = text_manager.create_texts(device, &[
            ("0", &Rect{ x: 100, y: 850, width: 100, height: 100 }),
            ("LOSERO", &Rect{ x: 150, y: 300, width: 200, height: 200 }),
        ]);

        let score_text = texts.remove(0);
        let end_text = texts.remove(0);

        let backdrop = Backdrop::new(device, command_pool, "background.png");

        let mut button_manager = ButtonManager::new(device, command_pool);

        let mut buttons = button_manager.create_buttons(device, &[(&Rect{x: 100, y: 100, width: 300, height: 100}, (255, 255, 255), "RESET")],
         &text_manager.get_text_renderer());

        let reset_button = buttons.remove(0);

        button_manager.add_button(&reset_button);
        button_manager.update(device);

        UserInterface {
            vertex_buffer: buffers.0,
            index_buffer: buffers.1,
            text_manager,
            score_text,
            end_text,
            game_state: GameState::RUNNING,
            score,
            backdrop,
            button_manager,
            reset_button,
            last_pressed: false
        }
    }

   

    fn initialize_buffers(
        device: &Device,
        command_pool: &CommandPool,
    ) -> (Buffer, Buffer) {
        let indices: [u16; 6] = [0, 1, 2, 1, 2, 3];
        let vertices: [f32; 8] = [0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32];

        let mut command_buffer = CommandBuffer::new(device, command_pool, false);

        command_buffer.begin(
            device,
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        
        let vertex_buffer = Buffer::new(
            device,
            &mut command_buffer,
            bytes_of(&vertices),
            BufferType::Vertex,
            false,
        );

        let index_buffer = Buffer::new(
            device,
            &mut command_buffer,
            bytes_of(&indices),
            BufferType::Index,
            false,
        );

        command_buffer.end(device);

        let fence = Fence::new(device, false);
        
        CommandBuffer::submit(device, &[command_buffer.get_command_buffer()], &[], &[], fence.get_fence());

        unsafe {


            device
                .get_ash_device()
                .wait_for_fences(&[fence.get_fence()], true, u64::MAX)
                .expect(
                    "Failed to wait for the fence for the initialization of UI command buffers",
                );
        }

        command_buffer.cleanup(device);

        (vertex_buffer, index_buffer)
    }

    fn handle_buttons(&mut self, window: &Window, board: &mut Board) {
        let mouse_state = window.get_window_handle().get_mouse_button(glfw::MouseButton::Button1);
        let is_pressed = mouse_state == glfw::Action::Press;

        if is_pressed == self.last_pressed || mouse_state == glfw::Action::Release{
            self.last_pressed = is_pressed;
            return;
        }

        let (x, y) = window.get_window_handle().get_cursor_pos();
        let mouse_pos = (x as u32, y as u32);


        if self.reset_button.is_pressed(mouse_pos) {
            board.reset_game();
        }


        self.last_pressed = is_pressed;
    }

    pub fn update(&mut self, state: GameState, window: &Window, board: &mut Board) {
        self.handle_buttons(window, board);

        self.game_state = state;
    }


    pub fn draw(
        &mut self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32,
        tetromino_instance_count: u32
    ) {
        self.backdrop.draw(device, render_pass, command_buffer, subpass_index, tetromino_instance_count, &self.vertex_buffer, &self.index_buffer);
        self.draw_texts(device, render_pass, command_buffer, subpass_index+1);
        self.button_manager.draw(device, command_buffer, &self.vertex_buffer, &self.index_buffer, render_pass, subpass_index+2);

    }

    fn draw_texts(
        &mut self,
        device: &Device,
        render_pass: &RenderPass,
        command_buffer: &CommandBuffer,
        subpass_index: u32) {

        self.text_manager.change_texts(device, &mut [(&mut self.score_text, &(*self.score.lock().expect("Failed to lock mutex womp womp").to_string()))]);

        self.text_manager.get_text_renderer().prepare_text_renderer(device, command_buffer, &self.vertex_buffer, &self.index_buffer,
             render_pass, subpass_index);
            
        self.score_text.draw(device, command_buffer, &self.text_manager.get_text_renderer(), render_pass);

        self.reset_button.draw_text(device, &self.text_manager.get_text_renderer(), command_buffer, render_pass);

        if matches!(self.game_state, GameState::END) {
            self.end_text.draw(device, command_buffer, &self.text_manager.get_text_renderer(), render_pass);
        }
    }




    pub fn get_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet<'a>; 2],  [Pin<Box<DescriptorInfo>>; 2]) {

        let (backdrop_write, mut backdrop_info) = self.backdrop.get_descriptor_write_sets(set);
        let (text_write, mut text_info) = self.text_manager.get_text_renderer().get_descriptor_write_sets(set);

        ([backdrop_write[0], text_write[0]], [backdrop_info.remove(0), text_info.remove(0)])
    }

    pub fn get_required_vertex_input_states() -> ([vk::PipelineVertexInputStateCreateInfo<'a>; 2], VertexInputData){
        let vertex_bindings = vec![
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 8,
                input_rate: vk::VertexInputRate::VERTEX,
            },

            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 8,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        ];

        let vertex_attributes = vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,

                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },

            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 1,

                format: vk::Format::R8_UINT,
                offset: 0,
            },

            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 1,

                format: vk::Format::R32_SFLOAT,
                offset: 4,
            },
 
        ];

        ([
            vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,

                vertex_attribute_description_count: 1,
                p_vertex_attribute_descriptions: &vertex_attributes[0],

                vertex_binding_description_count: 1,
                p_vertex_binding_descriptions: &vertex_bindings[0],

                ..Default::default()
            },
        
            vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,

                vertex_attribute_description_count: vertex_attributes.len() as u32,
                p_vertex_attribute_descriptions: vertex_attributes.as_ptr(),

                vertex_binding_description_count: vertex_bindings.len() as u32,
                p_vertex_binding_descriptions: vertex_bindings.as_ptr(),

                ..Default::default()
            },
        ], (vertex_bindings, vertex_attributes))

    }

    pub fn destroy(&mut self, device: &Device) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
        self.backdrop.destroy(device);
        self.score_text.destroy(device);
        self.end_text.destroy(device);
        self.text_manager.destroy(device);
        self.button_manager.destroy(device);
        self.reset_button.destroy(device);
    }
}
