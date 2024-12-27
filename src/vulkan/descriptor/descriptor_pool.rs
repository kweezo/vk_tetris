use super::super::*;
use ash::vk;

pub struct DescriptorPool {
    pool: vk::DescriptorPool,
}

impl DescriptorPool {
    fn create_pool_sizes(
        descriptor_type_sizes: &[(vk::DescriptorType, u32)],
    ) -> Vec<vk::DescriptorPoolSize> {
        let mut pool_sizes =
            Vec::<vk::DescriptorPoolSize>::with_capacity(descriptor_type_sizes.len());

        for type_size in descriptor_type_sizes {
            pool_sizes.push(vk::DescriptorPoolSize {
                ty: type_size.0,
                descriptor_count: type_size.1,
            });
        }

        pool_sizes
    }

    pub fn new(
        device: &core::Device,
        descriptor_type_sizes: &[(vk::DescriptorType, u32)],
    ) -> DescriptorPool {
        let pool_sizes = DescriptorPool::create_pool_sizes(descriptor_type_sizes);

        let create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,

            flags: vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND,

            max_sets: 1,

            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };

        let pool = unsafe {
            device
                .get_ash_device()
                .create_descriptor_pool(&create_info, None)
        }
        .expect("Failed to create a descriptor pool");

        DescriptorPool { pool }
    }

    pub fn get_pool(&self) -> vk::DescriptorPool {
        self.pool
    }
}
