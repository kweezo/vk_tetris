use ash::vk;
use ::image::open;
use super::*;
use super::core::*;

pub struct Texture{
    image: Image,
    sampler: vk::Sampler,
    image_view: vk::ImageView
}

impl Texture{
    pub fn new(path: &str, device: &Device, commad_buffer: &mut CommandBuffer) -> Option<Texture>{
        let image_raw =
        match open(path){
            Ok(img) => img.into_rgba8(),
            Err(e) =>  {
                eprintln!("WARNING: Failed to open image {path}, because {e}");
                return None;
            }
        };

        let data = unsafe{std::slice::from_raw_parts(image_raw.as_ptr(), (image_raw.width() * image_raw.height() * 4) as usize)};

        let image = Image::new(device, data, image_raw.width(), image_raw.height(), commad_buffer);
        let image_view = Texture::create_image_view(device, image.get_image());
        let sampler = Texture::create_sampler(device);

        Some(Texture{image: image, sampler: sampler, image_view: image_view})
    }


    fn create_sampler(device: &Device) -> vk::Sampler{
        let create_info = vk::SamplerCreateInfo{
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,

            mag_filter: vk::Filter::NEAREST,
            min_filter: vk::Filter::NEAREST,
            mipmap_mode: vk::SamplerMipmapMode::NEAREST,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,

            mip_lod_bias: 1.0f32,

            anisotropy_enable: false as u32,
            compare_enable: false as u32,

            min_lod: 0.0f32,
            max_lod: 1.0f32,

            ..Default::default()
        };


        let sampler = unsafe{device.get_ash_device().create_sampler(&create_info, None)}.expect("Failed to create a sampler");

        sampler
    }

    fn create_image_view(device: &Device, image: vk::Image) -> vk::ImageView{
        let create_info = vk::ImageViewCreateInfo{
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,

            image: image,

            view_type: vk::ImageViewType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            components: vk::ComponentMapping::default(),
            subresource_range: vk::ImageSubresourceRange{aspect_mask: vk::ImageAspectFlags::COLOR, base_array_layer: 0, base_mip_level: 0, layer_count: 1, level_count: 1},

            ..Default::default()
        };

        let image_view = unsafe{device.get_ash_device().create_image_view(&create_info, None)}.expect("Failed to create an image view");

        image_view
    }

    pub fn get_sampler(&self) -> vk::Sampler{
        self.sampler
    }

    pub fn get_image_view(&self) -> vk::ImageView{
        self.image_view
    }

    pub fn destroy(&mut self, device: &Device){
        self.image.destroy(device);
    }
}