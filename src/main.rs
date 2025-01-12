#![allow(dead_code)]
#![warn(clippy::pedantic)]

mod window;
mod types;

use ash::vk;

use descriptor::DescriptorSet;
use vulkan::*;

use game::{Board, UserInterface};
use window::*;

mod game;
mod vulkan;

macro_rules! device {
    ($x:ident) => {
        $x.get_device().get_ash_device()
    };
}

fn main() {
    let mut window = Window::new(1280, 720, "le title");

    let core = vulkan::Core::new(&window);

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

    let command_pool = CommandPool::new(
        core.get_device(),
        core.get_device().get_queue_family_index(),
        vk::CommandPoolCreateFlags::empty(),
    );

    let binding_sizes = [
        descriptor::DescriptorCreateInfo{descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, size: 1, binding : 6},
        descriptor::DescriptorCreateInfo{descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, size: 1, binding : 7},
        descriptor::DescriptorCreateInfo{descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, size: 3, binding: 1},
    ];

    let descriptor_set = DescriptorSet::new(core.get_device(), &binding_sizes);

    let (board_vertex_inputs, _board_vertex_input_data) = Board::get_required_vertex_input_states();
    let (ui_vertex_inputs, _ui_vertex_input_data) = UserInterface::get_required_vertex_input_states();

    let mut vertex_inputs = Vec::<vk::PipelineVertexInputStateCreateInfo>::with_capacity(board_vertex_inputs.len() + ui_vertex_inputs.len());
    vertex_inputs.extend_from_slice(&board_vertex_inputs);
    vertex_inputs.extend_from_slice(&ui_vertex_inputs);


    let mut render_pass = RenderPass::new(
        core.get_device(),
        &[tetromino_shader, backdrop_shader, text_shader],
        core.get_swapchain(),
        &vertex_inputs.as_slice(),
        descriptor_set.get_layout(),
    );
    let image_acquire_fence = Fence::new(core.get_device(), false);
    let render_finished_semaphore = Semaphore::new(core.get_device());

    let command_buffer = CommandBuffer::new(core.get_device(), &command_pool, false);

    let mut board = Board::new(
        core.get_device(),
        "tetromino_piece.png",
        &command_pool,
        (
            core.get_swapchain().get_swapchain_info().extent.width,
            core.get_swapchain().get_swapchain_info().extent.height,
        ),
    );

    let mut user_interface = UserInterface::new(core.get_device(), &command_pool, "background.png");

    while !window.get_window_handle().should_close() {
        window.get_glfw_context_mut().poll_events();

        board.update(window.get_events());

        let image_index: u32;

        unsafe {
            image_index = core
                .get_swapchain()
                .get_swapchain_info()
                .swapchain_device
                .acquire_next_image(
                    core.get_swapchain().get_swapchain_info().swapchain,
                    u64::MAX,
                    vk::Semaphore::null(),
                    image_acquire_fence.get_fence(),
                )
                .expect("Failed to acquire the next swapchain image")
                .0;

            device!(core)
                .wait_for_fences(&[image_acquire_fence.get_fence()], true, u64::MAX)
                .expect("Failed to wait for the image acquire fence");
            device!(core)
                .reset_fences(&[image_acquire_fence.get_fence()])
                .expect("Failed to reset the image acquisition fence");

            device!(core)
                .reset_command_pool(
                    command_pool.get_command_pool(),
                    vk::CommandPoolResetFlags::empty(),
                )
                .expect("Failed to reset the main command pool");
        }

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
            framebuffer: render_pass.get_framebuffer(image_index),
            render_pass: render_pass.get_render_pass(),
            render_area: vk::Rect2D {
                extent: core.get_swapchain().get_swapchain_info().extent,
                offset: vk::Offset2D { x: 0, y: 0 },
            },

            clear_value_count: 2,
            p_clear_values: [clear_color_value, clear_depth_value].as_ptr(),

            ..Default::default()
        };

        command_buffer.begin(
            core.get_device(),
            &vk::CommandBufferInheritanceInfo::default(),
            vk::CommandBufferUsageFlags::empty(),
        );

        unsafe {
            device!(core).cmd_bind_descriptor_sets(
                command_buffer.get_command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                render_pass.get_layout(),
                0,
                &[descriptor_set.get_set()],
                &[],
            );

            device!(core).cmd_begin_render_pass(
                command_buffer.get_command_buffer(),
                &begin_info,
                vk::SubpassContents::INLINE,
            );

            board.draw(core.get_device(), &render_pass, &command_buffer, 0);

            let (board_write_sets, _write_infos) = board.get_descriptor_write_sets(&descriptor_set);

            device!(core).cmd_next_subpass(
                command_buffer.get_command_buffer(),
                vk::SubpassContents::INLINE,
            );

            let (ui_write_sets, _write_infos) =
                user_interface.get_descriptor_write_sets(&descriptor_set);

            let (text_renderer_write_sets, _text_renderer_write_infos) = 
                user_interface.get_text_renderer_descriptor_sets(&descriptor_set);

            let mut write_sets = Vec::with_capacity(board_write_sets.len() + ui_write_sets.len() + text_renderer_write_sets.len());
            write_sets.extend_from_slice(&board_write_sets);
            write_sets.extend_from_slice(&ui_write_sets);
            write_sets.extend_from_slice(&text_renderer_write_sets);

            descriptor_set.update(core.get_device(), &write_sets.as_slice());

            user_interface.draw(core.get_device(), &render_pass, &command_buffer, 1);




            device!(core).cmd_end_render_pass(command_buffer.get_command_buffer());
        }

        command_buffer.end(core.get_device());

        CommandBuffer::submit(
            core.get_device(),
            &[command_buffer.get_command_buffer()],
            &[(
                board.get_transfer_semaphore(),
                vk::PipelineStageFlags::VERTEX_INPUT,
            )],
            &[render_finished_semaphore.get_semaphore()],
            vk::Fence::null(),
        );

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: [render_finished_semaphore.get_semaphore()].as_ptr(),

            swapchain_count: 1,
            p_swapchains: [core.get_swapchain().get_swapchain_info().swapchain].as_ptr(),
            p_image_indices: [image_index].as_ptr(),

            ..Default::default()
        };

        unsafe {
            core.get_swapchain()
                .get_swapchain_info()
                .swapchain_device
                .queue_present(core.get_device().get_queue(), &present_info)
                .expect("Failed to present image");
        }
    }

    unsafe {
        device!(core)
            .device_wait_idle()
            .expect("Failed to wait idle");
    }

    //TODO FIX

    board.destruct(core.get_device());
    user_interface.destroy(core.get_device());
    render_pass.destroy(core.get_device());
}
