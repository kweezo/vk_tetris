use ash::vk;

use crate::vulkan::shader::Shader;

use super::core::*;

pub struct Pipeline {
    pipeline: vk::Pipeline,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
}

impl Pipeline {
    fn create_pipeline_layout(device: &Device) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            set_layout_count: 0,
            push_constant_range_count: 0,
            ..Default::default()
        };

        let pipeline_layout = unsafe {
            device
                .get_ash_device()
                .create_pipeline_layout(&create_info, None)
        }
        .expect("Failed to create the pipeline layout");

        pipeline_layout
    }

    fn create_graphics_pipeline(
        device: &Device,
        layout: vk::PipelineLayout,
        shader: &Shader,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass
    ) -> vk::Pipeline {
        let shader_stages = shader.get_pipeline_stage_shader_info();

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            ..Default::default()
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: false as u32,
            ..Default::default()
        };

        let tessellation_state = vk::PipelineTessellationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0f32,
            y: 0f32,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0f32,
            max_depth: 1f32,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0i32, y: 0i32 },
            extent: extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            viewport_count: 1u32,
            p_viewports: &viewport,
            scissor_count: 1u32,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1f32,
            ..Default::default()
        };

        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            depth_test_enable: true as u32,
            ..Default::default()
        };

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState{
            blend_enable: false as u32,
            ..Default::default()
        };

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            logic_op_enable: false as u32,
            attachment_count: 1,
            p_attachments: &color_blend_attachment_state,
            ..Default::default()
        };

        let dynamic_states = vec![/*vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR*/];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            render_pass: render_pass,
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state,
            p_input_assembly_state: &input_assembly,
            p_tessellation_state: &tessellation_state,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterization_state,
            p_multisample_state: &multisample_state,
            p_depth_stencil_state: &depth_stencil_state,
            p_dynamic_state: &dynamic_state,
            p_color_blend_state: &color_blend_state,
            layout: layout,
            subpass: 0,
            ..Default::default()
        };

        let pipeline = unsafe {
            device.get_ash_device().create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            )
        }
        .expect("Failed to create the graphics pipeline")[0];

        pipeline
    }

    fn create_render_pass(device: &Device, format: vk::Format) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription {
            format: format,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass_description = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            ..Default::default()
        };

        let render_pass_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            attachment_count: 1,
            p_attachments: &color_attachment,
            subpass_count: 1,
            p_subpasses: &subpass_description,
            ..Default::default()
        };

        let render_pass = unsafe {
            device
                .get_ash_device()
                .create_render_pass(&render_pass_info, None)
        }
        .expect("Failed to create the render pass");

        render_pass
    }

    pub fn create_framebuffers(
        device: &Device,
        image_count: u32,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
    ) -> Vec<vk::Framebuffer> {
        let mut framebuffers = vec![vk::Framebuffer::null(); 3];


        for i in 0..image_count {
            let create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                render_pass: render_pass,
                attachment_count: 1,
                p_attachments: &image_views[i as usize],
                width: extent.width,
                height: extent.height,
                layers: 1,
                ..Default::default()
            };


            framebuffers[i as usize] = 
                unsafe {
                    device
                        .get_ash_device()
                        .create_framebuffer(&create_info, None)
                }
                .expect("Failed to create the swapchain framebuffers");
        }

        framebuffers
    }

    pub fn new(device: &Device, shader: &Shader, swapchain: &Swapchain) -> Pipeline {
        let layout = Pipeline::create_pipeline_layout(device);

        let render_pass =
            Pipeline::create_render_pass(device, swapchain.get_swapchain_info().format.format);

        let pipeline = Pipeline::create_graphics_pipeline(
            device,
            layout,
            shader,
            swapchain.get_swapchain_info().extent,
            render_pass
        );

        let framebuffers = Pipeline::create_framebuffers(
            device,
            swapchain.get_swapchain_info().image_count,
            swapchain.get_swapchain_info().extent,
            render_pass,
            swapchain.get_image_views(),
        );

        Pipeline {
            pipeline: pipeline,
            render_pass: render_pass,
            framebuffers: framebuffers,
        }
    }

    pub fn get_pipeline(&self) -> vk::Pipeline{
        self.pipeline
    }

    pub fn get_render_pass(&self) -> vk::RenderPass{
        self.render_pass
    }

    pub fn get_framebuffer(&self, index: u32) -> vk::Framebuffer{
        self.framebuffers[index as usize]
    }
}