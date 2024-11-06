use ash::{vk, khr};
use glm::ext::pi;

use crate::{vulkan::shader::Shader, Window};

use super::*;

pub struct Pipeline{
    pipeline: vk::Pipeline,
}

impl Pipeline{

    fn create_pipeline_layout(device: &Device) -> vk::PipelineLayout{
        let create_info = vk::PipelineLayoutCreateInfo{
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            set_layout_count: 0,
            push_constant_range_count: 0,
            ..Default::default()
        };
        
        let pipeline_layout = unsafe{device.get_ash_device().create_pipeline_layout(&create_info, None)}.expect("Failed to create the pipeline layout");

        pipeline_layout
    }

    //fn create_render_pass() {

    //    let subpasses = vk::subp

    //    //let create_info = vk::RenderPassCreateInfo{
    //    //    s_type: vk::StructureType::RENDER_PASS_CREATE_INFO
    //    //}
    //}

    fn create_graphics_pipeline(window: &Window, device: &Device, layout: vk::PipelineLayout, shader: &Shader, extent: vk::Extent2D) -> vk::Pipeline{
        let shader_stages = shader.get_pipeline_stage_shader_info();

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            ..Default::default()
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: false,
            ..Default::default()
        };

        let tessellation_state = vk::PipelineTessellationStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
            ..Default::default()
        };


        let viewport = vk::Viewport{
            x: 0f32,
            y: 0f32,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0f32,
            max_depth: 1f32
        };

        let scissor  = vk::Rect2D{
            offset: vk::Offset2D{x: 0i32, y: 0i32},
            extent: extent
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            viewport_count: 1u32,
            p_viewports: &viewport,
            scissor_count: 1u32,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            ..Default::default()
        };

        let multisample_state = vk::PipelineMultisampleStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            depth_test_enable: true as u32,
            ..Default::default()
        };

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            logic_op_enable: false as u32,
            ..Default::default()
        };

        let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo{
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo{
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
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
            layout: layout,
            subpass: 0,
            ..Default::default()
        };

        let pipeline = unsafe{device.get_ash_device().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_info), None)}.expect("Failed to create the graphics pipeline")[0];

        pipeline
    }

    fn create_render_pass(device: &Device, format: vk::Format){

        let color_attachment = vk::AttachmentDescription{
            format: format,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR
            ..Default::default()
        };

        let subpass_info = vk::RenderPassCreateInfo{
            vk::StructureType::RENDER_PASS_CREATE_INFO,
            
            ..Default::default()
        };
    }

    pub fn new(window: &Window, device: &Device, shader: &Shader, extent: vk::Extent2D, format: vk::Format) -> Pipeline{
        let layout = Pipeline::create_pipeline_layout(device);
        let pipeline = Pipeline::create_graphics_pipeline(window, device, layout, shader, extent);

        Pipeline{pipeline: pipeline}
    }
}