use descriptor::DescriptorSet;
use ash::vk;

use crate::*;

macro_rules! device {
    ($x:ident) => {
        $x.core.get_device().get_ash_device()
    };
}


pub struct Game {
    window: Window,

    core: Core,
    render_pass: RenderPass,

    command_pool: CommandPool,
    command_buffer: CommandBuffer,

    set: DescriptorSet,

    user_interface: UserInterface,
    board: Board,

    image_acquisition_fence: Fence,
    render_finish_semaphore: Semaphore,
}

impl Game {
    pub fn new() -> Game {
        let window = Window::new(720, 1280, "TETRIS");

        let core = Core::new(&window);

        let set = Game::create_descriptor_set(&core);
        let render_pass = Game::create_render_pass(&core, &set);

        let command_pool = CommandPool::new(
            core.get_device(),
            core.get_device().get_queue_family_index(),
            vk::CommandPoolCreateFlags::empty(),
        );

        let command_buffer = CommandBuffer::new(core.get_device(), &command_pool, false);

        let board = Board::new(
            core.get_device(),
            &command_pool,
            (
                core.get_swapchain().get_swapchain_info().extent.width,
                core.get_swapchain().get_swapchain_info().extent.height,
            ),
        );

        let user_interface = UserInterface::new(core.get_device(), &command_pool, board.get_score());

        Game::initialize_descriptor_set(&core, &set, &board, &user_interface);

        let user_interface = UserInterface::new(core.get_device(), &command_pool, board.get_score());


        let image_acquisition_fence = Fence::new(core.get_device(), false);
        let render_finish_semaphore = Semaphore::new(core.get_device());

        Game { window, core, render_pass, command_pool, command_buffer, set, user_interface, board, image_acquisition_fence, render_finish_semaphore }

    }

    fn load_shaders(core: &Core) -> Vec<Shader> {
        let tetromino_shader = vulkan::Shader::new(
            core.get_device(),
            String::from("shaders/bin/tetromino_vert.spv"),
            String::from("shaders/bin/tetromino_frag.spv"),
        );

        let backdrop_shader = vulkan::Shader::new(
            core.get_device(),
            String::from("shaders/bin/backdrop_vert.spv"),
            String::from("shaders/bin/backdrop_frag.spv"),
        );

        let text_shader = vulkan::Shader::new(
            core.get_device(),
            String::from("shaders/bin/text_vert.spv"),
            String::from("shaders/bin/text_frag.spv"),
        );

        vec![tetromino_shader, backdrop_shader, text_shader]
    }

    fn create_descriptor_set(core: &Core) -> DescriptorSet{
        let binding_sizes = [
            descriptor::DescriptorCreateInfo{descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, size: 1, binding : 6},
            descriptor::DescriptorCreateInfo{descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, size: 1, binding : 7},
            descriptor::DescriptorCreateInfo{descriptor_type: vk::DescriptorType::STORAGE_BUFFER, size: 1, binding : 8},
            descriptor::DescriptorCreateInfo{descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, size: 3, binding: 1},
        ];

        DescriptorSet::new(core.get_device(), &binding_sizes)
    }

    fn create_render_pass(core: &Core, descriptor_set: &DescriptorSet) -> RenderPass{
        let shaders = Game::load_shaders(core);

        let (board_vertex_inputs, _board_vertex_input_data) = Board::get_required_vertex_input_states();
        let (ui_vertex_inputs, _ui_vertex_input_data) = UserInterface::get_required_vertex_input_states();

        let mut vertex_inputs = Vec::<vk::PipelineVertexInputStateCreateInfo>::with_capacity(board_vertex_inputs.len() + ui_vertex_inputs.len());
        vertex_inputs.extend_from_slice(&ui_vertex_inputs);
        vertex_inputs.extend_from_slice(&board_vertex_inputs);

        RenderPass::new(
            core.get_device(),
            &shaders,
            core.get_swapchain(),
            &vertex_inputs.as_slice(),
            descriptor_set.get_layout(),
        )
    }

    fn initialize_descriptor_set(core: &Core, set: &DescriptorSet, board: &Board, user_interface: &UserInterface) {
        let (board_write_sets, _write_infos) = board.get_descriptor_write_sets(&set);

        let (ui_write_sets, _write_infos) =
            user_interface.get_descriptor_write_sets(&set);

        let mut write_sets = Vec::with_capacity(board_write_sets.len() + ui_write_sets.len());
        write_sets.extend_from_slice(&board_write_sets);
        write_sets.extend_from_slice(&ui_write_sets);

        set.update(core.get_device(), &write_sets.as_slice());
    }

    fn update_descriptor_set(&self) {
        if self.board.instance_buffer_exists() {
            return;
        }

        let (board_write_sets, _write_infos) = self.board.get_instance_descriptor_write_sets(&self.set);

        let mut write_sets = Vec::with_capacity(board_write_sets.len());

        write_sets.extend_from_slice(&board_write_sets);
        self.set.update(self.core.get_device(), &write_sets.as_slice());
    }

    fn update(&mut self) {
        self.window.get_glfw_context_mut().poll_events();

        self.update_descriptor_set();

        self.board.update(self.window.get_events());
        self.user_interface.update(self.board.get_game_state());
    }

    fn get_image_index(&self) -> u32 {
        unsafe{


            device!(self)
                .wait_for_fences(&[self.image_acquisition_fence.get_fence()], true, u64::MAX)
                .expect("Failed to wait for the image acquisition fence");
            device!(self)
                .reset_fences(&[self.image_acquisition_fence.get_fence()])
                .expect("Failed to reset the image acquisition fence");

            self.core
            .get_swapchain()
            .get_swapchain_info()
            .swapchain_device
            .acquire_next_image(
                self.core.get_swapchain().get_swapchain_info().swapchain,
                u64::MAX,
                vk::Semaphore::null(),
                self.image_acquisition_fence.get_fence(),
            )
            .expect("Failed to acquire the next swapchain image")
            .0
        }
    }

    fn reset_command_pool(&self) {
        unsafe{
            device!(self)
            .reset_command_pool(
                self.command_pool.get_command_pool(),
                vk::CommandPoolResetFlags::empty(),
            )
            .expect("Failed to reset the main command pool");
        }
    }

    fn begin_command_buffer(&self, image_index: u32) {
        let clear_color_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            },
        };

        let clear_depth_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0f32,
                stencil: 0,
            },
        };

        let begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            framebuffer: self.render_pass.get_framebuffer(image_index),
            render_pass: self.render_pass.get_render_pass(),
            render_area: vk::Rect2D {
                extent: self.core.get_swapchain().get_swapchain_info().extent,
                offset: vk::Offset2D { x: 0, y: 0 },
            },

            clear_value_count: 2,
            p_clear_values: [clear_color_value, clear_depth_value].as_ptr(),

            ..Default::default()
        };

        self.command_buffer.begin(
            self.core.get_device(),
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        unsafe {
            device!(self).cmd_bind_descriptor_sets(
                self.command_buffer.get_command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                self.render_pass.get_layout(),
                0,
                &[self.set.get_set()],
                &[],
            );

            device!(self).cmd_begin_render_pass(
                self.command_buffer.get_command_buffer(),
                &begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    fn end_command_buffer_and_present(&self, image_index: u32) {
        unsafe{
            device!(self).cmd_end_render_pass(self.command_buffer.get_command_buffer());
        }

        self.command_buffer.end(self.core.get_device());

        CommandBuffer::submit(
            self.core.get_device(),
            &[self.command_buffer.get_command_buffer()],
            &[(
                self.board.get_transfer_semaphore(),
                vk::PipelineStageFlags::VERTEX_INPUT,
            )],
            &[self.render_finish_semaphore.get_semaphore()],
            vk::Fence::null(),
        );

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: [self.render_finish_semaphore.get_semaphore()].as_ptr(),

            swapchain_count: 1,
            p_swapchains: [self.core.get_swapchain().get_swapchain_info().swapchain].as_ptr(),
            p_image_indices: [image_index].as_ptr(),

            ..Default::default()
        };

        unsafe{
            self.core.get_swapchain()
                .get_swapchain_info()
                .swapchain_device
                .queue_present(self.core.get_device().get_queue(), &present_info)
                .expect("Failed to present image");
        }

    }

    fn render(&mut self) {
        self.reset_command_pool();

        let image_index = self.get_image_index();

        self.begin_command_buffer(image_index);

        self.user_interface.draw(self.core.get_device(), &self.render_pass, &self.command_buffer, 0, self.board.get_tetromino_instance_count());

        unsafe{
            device!(self).cmd_next_subpass(
                self.command_buffer.get_command_buffer(),
                vk::SubpassContents::INLINE,
            );
        }

        self.board.draw(self.core.get_device(), &self.render_pass, &self.command_buffer, 2);

        self.end_command_buffer_and_present(image_index);
    }

    pub fn game_loop(&mut self) {
        while !self.window.get_window_handle().should_close() {
            self.update();
            self.render();
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        unsafe {
            device!(self)
                .device_wait_idle()
                .expect("Failed to wait idle");
        }

        self.board.destruct(self.core.get_device());
        self.user_interface.destroy(self.core.get_device());
        self.render_pass.destroy(self.core.get_device());

    }
}