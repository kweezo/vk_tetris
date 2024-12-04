#![allow(dead_code)]

mod window;
use ash::vk;
use window::*;

use bytemuck::bytes_of;

use glm::*;

mod vulkan;

macro_rules! device {
    ($x:ident) => {
       $x.get_device().get_ash_device() 
    };
}

#[repr(align(32))]
struct Transform{
    v1: Vec2,
    _padding1: [u8; 8],
    v2: Vec2,
    _padding2: [u8; 8]
}

fn main() {
    let mut window = Window::new(1280, 720, "le title");

    let mut core = vulkan::Core::new(&window);
    let shader = vulkan::Shader::new(
        core.get_device(),
        String::from("shaders/bin/triangle_vert.spv"),
        String::from("shaders/bin/triangle_frag.spv"),
    );
    
    let binding_description = vk::VertexInputBindingDescription{
        binding: 0,
        stride: 24,
        input_rate: vk::VertexInputRate::VERTEX
    };

    let vertex_input = [
        vk::VertexInputAttributeDescription{
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0
        },
        vk::VertexInputAttributeDescription{
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 8
        }
    ];

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo{
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,

        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: &binding_description,

        vertex_attribute_description_count: vertex_input.len() as u32,
        p_vertex_attribute_descriptions: vertex_input.as_ptr(),

        ..Default::default()
    };

    let binding_size = (vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, 1);
    let descriptor_set = vulkan::descriptor::DescriptorSet::new(core.get_device(), std::slice::from_ref(&binding_size));

    let command_pool = vulkan::CommandPool::new(core.get_device(), core.get_device().get_queue_family_index(), vk::CommandPoolCreateFlags::empty());
    let command_buffer = vulkan::CommandBuffer::new(core.get_device(), &command_pool);
    let render_pass = vulkan::RenderPass::new(core.get_device(), &shader, &core.get_swapchain(),
     &vertex_input_state, descriptor_set.get_layout());

    
    let image_available_semaphore: vulkan::Semaphore = vulkan::Semaphore::new(core.get_device());
    let image_available_fence: vulkan::Fence = vulkan::Fence::new(core.get_device(), true);

    let render_finished_semaphore: vulkan::Semaphore = vulkan::Semaphore::new(core.get_device());

    let vertex_data: [f32; 24] = [
        -0.5, -0.5,     0.0, 0.0, 0.0,  0.0/*padding */,
        -0.5,  0.5,     0.0, 0.0, 0.0,  0.0,
         0.5, -0.5,     0.0, 0.0, 0.0,  0.0,
         0.5,  0.5,     0.0, 0.0, 0.0,  0.0,
    ];

    let index_data: [u16; 6] = [
        0, 1, 2,
        1, 2, 3
    ];

    let transform = Transform{
        v1: Vector2::new(1f32, 0f32),
        v2: Vector2::new(0f32, 1f32),

        _padding1: [0; 8],
        _padding2: [0; 8]
    };

    let mut vertex_buffer = vulkan::Buffer::new(core.get_device_mut(), &command_buffer, bytes_of(&vertex_data), vulkan::BufferType::VERTEX);

    unsafe{
        device!(core).reset_command_pool(command_pool.get_command_pool(), vk::CommandPoolResetFlags::empty()).expect("Failed to reset the command pool");
    }
    let mut index_buffer = vulkan::Buffer::new(core.get_device_mut(), &command_buffer, bytes_of(&index_data), vulkan::BufferType::INDEX);

    unsafe{
        device!(core).reset_command_pool(command_pool.get_command_pool(), vk::CommandPoolResetFlags::empty()).expect("Failed to reset the command pool");
    }
     
    
    let mut uniform_buffer = vulkan::Buffer::new(core.get_device_mut(), &command_buffer, unsafe{std::slice::from_raw_parts(&transform as *const _ as *const u8, std::mem::size_of_val(&transform))}, vulkan::BufferType::UNIFORM);

    let descriptor_buffer_info = vk::DescriptorBufferInfo{
        buffer: uniform_buffer.get_buffer(),
        offset: 0,
        range: vk::WHOLE_SIZE
    };

    let write_set = descriptor_set.create_write_set(&vulkan::descriptor::DescriptorInfo::Buffer(std::slice::from_ref(&descriptor_buffer_info)),
     vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, 0, 1);


    unsafe{
        device!(core).update_descriptor_sets(std::slice::from_ref(&write_set), &[]);
    }

    unsafe{
        device!(core).reset_command_pool(command_pool.get_command_pool(), vk::CommandPoolResetFlags::empty()).expect("Failed to reset the command pool");
    }

    let mut img_1 = vulkan::Image::with_path("i.png", core.get_device_mut(), &command_buffer).unwrap();


    while !window.get_window_handle().should_close() {
        unsafe{
            device!(core).wait_for_fences(std::slice::from_ref(&image_available_fence.get_fence()), true, std::u64::MAX).expect("Failed to wait for fences");
            device!(core).reset_fences(std::slice::from_ref(&image_available_fence.get_fence())).expect("Failed to reset fences");

            let current_frame = core.get_swapchain().get_swapchain_info().swapchain_device.acquire_next_image
                (core.get_swapchain().get_swapchain_info().swapchain, std::u64::MAX, image_available_semaphore.get_semaphore(), image_available_fence.get_fence())
                .expect("Failed to acquire the next image from the swapchain").0;
        



            device!(core).reset_command_pool(command_pool.get_command_pool(), vk::CommandPoolResetFlags::empty()).expect("Failed to reset the command pool");

            command_buffer.begin(core.get_device());

        


            let render_area = vk::Rect2D{
                offset: vk::Offset2D{x: 0, y: 0},
                extent: core.get_swapchain().get_swapchain_info().extent
            };

            let clear_value = vk::ClearValue{
                color: vk::ClearColorValue{
                    float32: [1.0, 0.0, 1.0, 1.0]
                }
            };

            let render_pass_begin_info = vk::RenderPassBeginInfo{
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                render_pass: render_pass.get_render_pass(),
                framebuffer: render_pass.get_framebuffer(current_frame),
                render_area: render_area,
                clear_value_count: 1,
                p_clear_values: &clear_value,
                ..Default::default()
            };


            device!(core).cmd_begin_render_pass(command_buffer.get_command_buffer(), &render_pass_begin_info, vk::SubpassContents::INLINE);

            device!(core).cmd_bind_pipeline(command_buffer.get_command_buffer(), vk::PipelineBindPoint::GRAPHICS, render_pass.get_pipeline());

            let offset: vk::DeviceSize = 0;

            device!(core).cmd_bind_descriptor_sets(command_buffer.get_command_buffer(), vk::PipelineBindPoint::GRAPHICS, render_pass.get_layout(), 0, std::slice::from_ref(&descriptor_set.get_set()), std::slice::from_ref(&(offset as u32)));

            let push_constants: [u8; 128] = [0; 128];
            device!(core).cmd_push_constants(command_buffer.get_command_buffer(), render_pass.get_layout(), vk::ShaderStageFlags::ALL, offset as u32, &push_constants);

            device!(core).cmd_bind_vertex_buffers(command_buffer.get_command_buffer(), 0, std::slice::from_ref(&vertex_buffer.get_buffer()), std::slice::from_ref(&offset));
            device!(core).cmd_bind_index_buffer(command_buffer.get_command_buffer(), index_buffer.get_buffer(), offset, vk::IndexType::UINT16);
            device!(core).cmd_draw_indexed(command_buffer.get_command_buffer(), index_data.len() as u32, 1, 0, 0, 0);

            device!(core).cmd_end_render_pass(command_buffer.get_command_buffer());

            command_buffer.end(core.get_device());

            let wait_dst_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

            let submit_info = vk::SubmitInfo{
                s_type: vk::StructureType::SUBMIT_INFO,
                p_command_buffers: &command_buffer.get_command_buffer(),
                command_buffer_count: 1,

                p_wait_semaphores: &image_available_semaphore.get_semaphore(),
                p_wait_dst_stage_mask: &wait_dst_stage_mask,
                wait_semaphore_count: 1,
                
                 
                p_signal_semaphores: &render_finished_semaphore.get_semaphore(),
                signal_semaphore_count: 1,

                ..Default::default()
            };


            let queue = &mut core.get_device_mut().get_queue();

            device!(core).queue_submit(*queue, std::slice::from_ref(&submit_info), vk::Fence::null())
            .expect("Failed to submit the command buffer");
        
            let present_info = vk::PresentInfoKHR {
                s_type: vk::StructureType::PRESENT_INFO_KHR,
                swapchain_count: 1,
                p_swapchains: &core.get_swapchain().get_swapchain_info().swapchain,
                p_image_indices: &current_frame,

                wait_semaphore_count: 1,
                p_wait_semaphores: &render_finished_semaphore.get_semaphore(),

                ..Default::default()
            };
        
            core.get_swapchain().get_swapchain_info().swapchain_device.queue_present(*queue, &present_info).
                expect("Failed to present the queue");


            window.get_glfw_context_mut().poll_events();
        }
    }

    unsafe{device!(core).device_wait_idle()}.expect("Failed to device wait idle");

    img_1.destroy(core.get_device());
    uniform_buffer.destroy(core.get_device());
    vertex_buffer.destroy(core.get_device());
    index_buffer.destroy(core.get_device());
}
