#![allow(dead_code)]

mod window;
use ash::vk;
use window::*;

mod vulkan;

fn main() {
    let mut window = Window::new(1280, 720, "le title");

    let mut core = vulkan::Core::new(&window);
    let shader = vulkan::Shader::new(
        core.get_device(),
        String::from("shaders/bin/triangle_vert.spv"),
        String::from("shaders/bin/triangle_frag.spv"),
    );
    

    let command_pool = vulkan::CommandPool::new(core.get_device(), core.get_device().get_queue_family_index());
    let command_buffer = vulkan::CommandBuffer::new(core.get_device(), &command_pool);
    let pipeline = vulkan::Pipeline::new(core.get_device(), &shader, &core.get_swapchain());

    let fence = vulkan::Fence::new(&core.get_device());
    let present_semaphore = vulkan::Semaphore::new(&core.get_device());

    let mut current_frame = 0u32;

    while !window.get_window_handle().should_close() {
        unsafe{
           current_frame = core.get_swapchain().get_swapchain_info().swapchain_device.acquire_next_image
                (core.get_swapchain().get_swapchain_info().swapchain, std::u64::MAX, vk::Semaphore::null(), fence.get_fence())
                .expect("Failed to acquire the next image from the swapchain").0;


            core.get_device().get_ash_device().wait_for_fences(std::slice::from_ref(&fence.get_fence()), true, std::u64::MAX).expect("Failed to wait for all the fences (like what the fuck lmao)");
            core.get_device().get_ash_device().reset_fences(std::slice::from_ref(&fence.get_fence())).expect("Failed to reset the fences");


            core.get_device().get_ash_device().reset_command_pool(command_pool.get_command_pool(), vk::CommandPoolResetFlags::empty()).expect("Failed to reset the command pool");

            command_buffer.begin(core.get_device());

        


            let render_area = vk::Rect2D{
                offset: vk::Offset2D{x: 0, y: 0},
                extent: core.get_swapchain().get_swapchain_info().extent
            };

            let clear_value = vk::ClearValue::default();

            let render_pass_begin_info = vk::RenderPassBeginInfo{
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                render_pass: pipeline.get_render_pass(),
                framebuffer: pipeline.get_framebuffer(current_frame),
                render_area: render_area,
                clear_value_count: 1,
                p_clear_values: &clear_value,
                ..Default::default()
            };

            core.get_device().get_ash_device().cmd_begin_render_pass(command_buffer.get_command_buffer(), &render_pass_begin_info, vk::SubpassContents::INLINE);

            core.get_device().get_ash_device().cmd_bind_pipeline(command_buffer.get_command_buffer(), vk::PipelineBindPoint::GRAPHICS, pipeline.get_pipeline());

            core.get_device().get_ash_device().cmd_draw(command_buffer.get_command_buffer(), 3, 1, 0, 0);

            core.get_device().get_ash_device().cmd_end_render_pass(command_buffer.get_command_buffer());

            command_buffer.end(core.get_device());


            let submit_info = vk::SubmitInfo{
                s_type: vk::StructureType::SUBMIT_INFO,
                ..Default::default()
            };

        
            let queue = &mut core.get_device_mut().get_queue();

            core.get_device().get_ash_device().queue_submit(*queue, std::slice::from_ref(&submit_info), vk::Fence::null()).expect("Failed to submit the command buffer");
        
            let present_info = vk::PresentInfoKHR {
                s_type: vk::StructureType::PRESENT_INFO_KHR,
                swapchain_count: 1,
                p_swapchains: &core.get_swapchain().get_swapchain_info().swapchain,
                p_image_indices: &current_frame,
                ..Default::default()
            };
        
            core.get_swapchain().get_swapchain_info().swapchain_device.queue_present(*queue, &present_info).
                expect("Failed to present the queue");


            window.get_glfw_context_mut().poll_events();
        }
    }
}
