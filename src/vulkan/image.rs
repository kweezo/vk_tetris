use super::{core::*, *};
use ash::vk;
use vk_mem::Alloc;

pub struct TransitionInfo {
    pub old_layout: vk::ImageLayout,
    pub new_layout: vk::ImageLayout,
    pub src_access: vk::AccessFlags,
    pub dst_access: vk::AccessFlags,
    pub aspect_mask: vk::ImageAspectFlags,
    pub src_stage: vk::PipelineStageFlags,
    pub dst_stage: vk::PipelineStageFlags,
}
#[derive(Debug)]
pub enum Type {
    SAMPLED,
    DEPTH,
    COLOR
}

pub struct Image {
    image: vk::Image,
    allocation: vk_mem::Allocation,
    destroyed: bool
}

impl Image {
    pub fn new_empty(
        device: &Device,
        width: u32,
        height: u32,
        image_type: Type,
        format: vk::Format,
        samples: vk::SampleCountFlags
    ) -> Image {
        let (image, allocation) = Image::create_image(device, width, height, image_type, format, samples);

        Image { image, allocation, destroyed: false }
    }

    pub fn new(
        device: &Device,
        commad_buffer: &mut CommandBuffer,
        width: u32,
        height: u32,
        image_type: Type,
        format: vk::Format,
        samples: vk::SampleCountFlags,
        data: &[u8],
    ) -> Image {
        let image = Image::new_empty(device, width, height, image_type, format, samples);

        Image::copy_data_to_image(
            device,
            image.get_image(),
            commad_buffer,
            data,
            width,
            height,
        );

        image
    }

    fn create_image(
        device: &Device,
        width: u32,
        height: u32,
        image_type: Type,
        format: vk::Format,
        samples: vk::SampleCountFlags,
    ) -> (vk::Image, vk_mem::Allocation) {
        let image_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            image_type: vk::ImageType::TYPE_2D,

            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },

            format,

            usage: vk::ImageUsageFlags::TRANSFER_DST
                | match image_type {
                    Type::SAMPLED => vk::ImageUsageFlags::SAMPLED,
                    Type::DEPTH => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                    Type::COLOR => vk::ImageUsageFlags::COLOR_ATTACHMENT
                },

            mip_levels: 1,
            array_layers: 1,

            samples: samples,

            tiling: vk::ImageTiling::OPTIMAL,

            sharing_mode: vk::SharingMode::EXCLUSIVE,

            initial_layout: vk::ImageLayout::UNDEFINED,

            ..Default::default()
        };

        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::Auto,
            required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ..Default::default()
        };

        let (image, allocation) = unsafe {
            device
                .get_allocator()
                .create_image(&image_info, &allocation_info)
        }
        .expect("Failed to create a vulkan image with VMA");

        (image, allocation)
    }

    fn copy_data_to_image(
        device: &Device,
        image: vk::Image,
        commad_buffer: &mut CommandBuffer,
        data: &[u8],
        width: u32,
        height: u32,
    ) {
        let (staging_buffer, staging_allocation) = Buffer::setup_staging_buffer(device, data);

        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },

            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },

            ..Default::default()
        };

        {
            let info = TransitionInfo {
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                src_access: vk::AccessFlags::NONE,
                dst_access: vk::AccessFlags::TRANSFER_WRITE,
                aspect_mask: vk::ImageAspectFlags::COLOR,
                src_stage: vk::PipelineStageFlags::TOP_OF_PIPE,
                dst_stage: vk::PipelineStageFlags::TRANSFER,
            };

            Image::transition_layout(device, commad_buffer, image, &info);
        }

        unsafe {
            device.get_ash_device().cmd_copy_buffer_to_image(
                commad_buffer.get_command_buffer(),
                staging_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        {
            let info = TransitionInfo {
                old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                src_access: vk::AccessFlags::TRANSFER_WRITE,
                dst_access: vk::AccessFlags::NONE,
                aspect_mask: vk::ImageAspectFlags::COLOR,
                src_stage: vk::PipelineStageFlags::TRANSFER,
                dst_stage: vk::PipelineStageFlags::FRAGMENT_SHADER,
            };

            Image::transition_layout(device, commad_buffer, image, &info);
        }

        commad_buffer.add_to_cleanup_list(staging_buffer, staging_allocation);
    }

    fn transition_layout(
        device: &Device,
        commad_buffer: &CommandBuffer,
        image: vk::Image,
        info: &TransitionInfo,
    ) {
        let image_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            src_access_mask: info.src_access,
            dst_access_mask: info.dst_access,
            old_layout: info.old_layout,
            new_layout: info.new_layout,

            image,

            subresource_range: vk::ImageSubresourceRange {
                layer_count: 1,
                level_count: 1,
                base_array_layer: 0,
                base_mip_level: 0,
                aspect_mask: info.aspect_mask,
            },

            ..Default::default()
        };
        
        unsafe {
            device.get_ash_device().cmd_pipeline_barrier(
                commad_buffer.get_command_buffer(),
                info.src_stage,
                info.dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            )
        };
    }

    pub fn get_image(&self) -> vk::Image {
        self.image
    }

    pub fn destroy(&mut self, device: &Device) {
        self.destroyed = true;

        unsafe {
            device
                .get_allocator()
                .destroy_image(self.image, &mut self.allocation)
        };
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        assert!(self.destroyed, "VMA image not freed before destruction");
    }
}
