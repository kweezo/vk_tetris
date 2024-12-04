use ash::vk;
use vk_mem::Alloc;
use super::{core::*, *};
use ::image::open;

pub struct Image{
    image: vk::Image,
    allocation: vk_mem::Allocation,
}

impl Image{
    pub fn new(device: &mut Device, data: &[u8], width: u32, height: u32, commad_buffer: &CommandBuffer) -> Image{
        let (image, allocation) = Image::create_image(device, width, height);

        Image::copy_data_to_image(device, image, commad_buffer, data, width, height);

        Image { image: image, allocation: allocation }
    }

    pub fn with_path(path: &str, device: &mut Device, commad_buffer: &CommandBuffer) -> Option<Image>{
        let image =
        match open(path){
            Ok(img) => img.into_rgba8(),
            Err(e) =>  {
                eprintln!("WARNING: Failed to open image {path}, because {e}");
                return None;
            }
        };

        let data = unsafe{std::slice::from_raw_parts(image.as_ptr(), (image.width() * image.height() * 4) as usize)};

        Some(Image::new(device, data, image.width(), image.height(), commad_buffer))
    }

    fn create_image(device: &mut Device, width: u32, height: u32) -> (vk::Image, vk_mem::Allocation){
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

    fn copy_data_to_image(device: &mut Device, image: vk::Image, commad_buffer: &CommandBuffer, data: &[u8], width: u32, height: u32){
        let (staging_buffer, _staging_allocation) = Buffer::setup_staging_buffer(&device, data);

        let region =  vk::BufferImageCopy{
            buffer_offset: 0,
            image_subresource: vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0 ,base_array_layer: 0, layer_count: 1 },

            image_extent: vk::Extent3D{width: width, height: height, depth: 1},

            ..Default::default()
        };

        commad_buffer.begin(device);

        unsafe{
           device.get_ash_device().cmd_copy_buffer_to_image(commad_buffer.get_command_buffer(), staging_buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, std::slice::from_ref(&region));
        }

        commad_buffer.end(device);

        let submit_info = vk::SubmitInfo{
            s_type: vk::StructureType::SUBMIT_INFO,
            command_buffer_count: 1,
            p_command_buffers: &commad_buffer.get_command_buffer(),
            ..Default::default()
        };

        let fence = Fence::new(device, false);

        unsafe{
            let queue = device.get_queue();

            device.get_ash_device().queue_submit(queue, std::slice::from_ref(&submit_info), fence.get_fence()).expect("Failed to submit a command buffer update command");
            device.get_ash_device().wait_for_fences(std::slice::from_ref(&fence.get_fence()), true, std::u64::MAX).expect("Failed to wait for the command update buffer submission");
        }
        
    }

    fn transition_layout(image: vk::Image, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, aspect_mask: vk::ImageAspectFlags){

    }

    pub fn destroy(&mut self, device: &Device){
        unsafe{device.get_allocator().destroy_image(self.image, &mut self.allocation)};
    }
}