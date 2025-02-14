use super::core::*;
use super::*;
use png;
use ash::vk;
use std::fs::File;

pub struct Texture {
    image: Image,
    sampler: vk::Sampler,
    image_view: vk::ImageView,
}

impl Texture {
    pub fn new(path: &str, device: &Device, command_buffer: &mut CommandBuffer, multisampling: bool) -> Option<Texture> {

        let decoder = png::Decoder::new(File::open(path).expect("Failed to open texture path"));

        let mut reader = decoder.read_info().expect("Failed to parse the png");

        let mut data_raw = vec![0; reader.output_buffer_size()];

        let info = reader.next_frame(&mut data_raw).unwrap();

        let data = &data_raw[..info.buffer_size()]; 

        let image = Image::new(
            device,
            command_buffer,
            info.width,
            info.height,
            image::Type::SAMPLED,
            vk::Format::R8G8B8A8_SRGB,
            match multisampling {
                true => vk::SampleCountFlags::TYPE_4,
                false => vk::SampleCountFlags::TYPE_1,
            },
            data,
        );

        let image_view =
            Texture::create_image_view(device, image.get_image(), vk::Format::R8G8B8A8_SRGB);
        let sampler = Texture::create_sampler(device);

        Some(Texture {
            image,
            sampler,
            image_view,
        })
    }

    pub fn new_raw_data(
        device: &Device,
        command_buffer: &mut CommandBuffer,
        data: &[u8],
        width: u32,
        height: u32,
        format: vk::Format,
        multisampling: bool
    ) -> Option<Texture> {
        let image = Image::new(
            device,
            command_buffer,
            width,
            height,
            image::Type::SAMPLED,
            format,
            match multisampling {
                true => vk::SampleCountFlags::TYPE_4,
                false => vk::SampleCountFlags::TYPE_1,
            },
            data,
        );

        let image_view = Texture::create_image_view(device, image.get_image(), format);
        let sampler = Texture::create_sampler(device);

        Some(Texture {
            image,
            sampler,
            image_view,
        })
    }

    fn create_sampler(device: &Device) -> vk::Sampler {
        let create_info = vk::SamplerCreateInfo {
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

        let sampler = unsafe { device.get_ash_device().create_sampler(&create_info, None) }
            .expect("Failed to create a sampler");

        sampler
    }

    fn create_image_view(device: &Device, image: vk::Image, format: vk::Format) -> vk::ImageView {
        let create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,

            image,

            view_type: vk::ImageViewType::TYPE_2D,
            format: format,
            components: vk::ComponentMapping::default(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_array_layer: 0,
                base_mip_level: 0,
                layer_count: 1,
                level_count: 1,
            },

            ..Default::default()
        };

        let image_view = unsafe {
            device
                .get_ash_device()
                .create_image_view(&create_info, None)
        }
        .expect("Failed to create an image view");

        image_view
    }

    pub fn get_sampler(&self) -> vk::Sampler {
        self.sampler
    }

    pub fn get_image_view(&self) -> vk::ImageView {
        self.image_view
    }

    pub fn destroy(&mut self, device: &Device) {
        self.image.destroy(device);
    }
}
