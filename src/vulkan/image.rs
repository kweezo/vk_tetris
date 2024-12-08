use ash::vk;
use vk_mem::Alloc;
use super::{core::*, *};

pub struct Image{
    image: vk::Image,
    allocation: vk_mem::Allocation,
}

impl Image{
    pub fn new(device: &Device, data: &[u8], width: u32, height: u32, commad_buffer: &mut CommandBuffer) -> Image{
        let (image, allocation) = Image::create_image(device, width, height);

        Image::copy_data_to_image(device, image, commad_buffer, data, width, height);

        Image { image: image, allocation: allocation }
    }

    

    fn create_image(device: &Device, width: u32, height: u32) -> (vk::Image, vk_mem::Allocation){
        let image_info = vk::ImageCreateInfo{
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            image_type: vk::ImageType::TYPE_2D,

            extent: vk::Extent3D{width: width, height: height, depth: 1},

            format: vk::Format::R8G8B8A8_SRGB,

            mip_levels: 1,
            array_layers: 1,

            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,

            sharing_mode: vk::SharingMode::EXCLUSIVE,

            initial_layout: vk::ImageLayout::UNDEFINED,

            ..Default::default()
        };

        let allocation_info = vk_mem::AllocationCreateInfo{
            usage: vk_mem::MemoryUsage::Auto,
            required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ..Default::default()
        };

       let (image, allocation) = unsafe{device.get_allocator().create_image(&image_info, &allocation_info)}.expect("Failed to create a vulkan image with VMA");


       (image, allocation)
    }

    fn copy_data_to_image(device: &Device, image: vk::Image, commad_buffer: &mut CommandBuffer, data: &[u8], width: u32, height: u32){
        let (staging_buffer, staging_allocation) = Buffer::setup_staging_buffer(&device, data);

        let region =  vk::BufferImageCopy{
            buffer_offset: 0,
            image_subresource: vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0 ,base_array_layer: 0, layer_count: 1 },

            image_extent: vk::Extent3D{width: width, height: height, depth: 1},

            ..Default::default()
        };

        Image::transition_layout(device, commad_buffer, image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
             vk::AccessFlags::NONE, vk::AccessFlags::TRANSFER_WRITE, vk::ImageAspectFlags::COLOR, vk::PipelineStageFlags::TOP_OF_PIPE,
              vk::PipelineStageFlags::TRANSFER);

        unsafe{
           device.get_ash_device().cmd_copy_buffer_to_image(commad_buffer.get_command_buffer(), staging_buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
        }

        Image::transition_layout(device, commad_buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
             vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::NONE, vk::ImageAspectFlags::COLOR, vk::PipelineStageFlags::TRANSFER,
              vk::PipelineStageFlags::FRAGMENT_SHADER);


        commad_buffer.add_to_cleanup_list(staging_buffer, staging_allocation);
    }

    fn transition_layout(device: &Device, commad_buffer: &CommandBuffer, image: vk::Image, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout,
         src_access: vk::AccessFlags, dst_access: vk::AccessFlags, aspect_mask: vk::ImageAspectFlags, src_stage: vk::PipelineStageFlags, dst_stage: vk::PipelineStageFlags){

        let image_barrier = vk::ImageMemoryBarrier{
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            src_access_mask: src_access,
            dst_access_mask: dst_access,
            old_layout: old_layout,
            new_layout: new_layout,

            image: image,

            subresource_range: vk::ImageSubresourceRange{layer_count: 1, level_count: 1, base_array_layer: 0, base_mip_level: 0, aspect_mask: aspect_mask},

            ..Default::default()
        };

        unsafe{device.get_ash_device().cmd_pipeline_barrier(commad_buffer.get_command_buffer(),
         src_stage, dst_stage,
          vk::DependencyFlags::empty(), &[], &[], &[image_barrier])};
    }

    pub fn get_image(&self) -> vk::Image{
        self.image
    }

    pub fn destroy(&mut self, device: &Device){
        unsafe{device.get_allocator().destroy_image(self.image, &mut self.allocation)};
    }
}