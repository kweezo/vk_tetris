use super::super::*;
use ash::vk;

pub struct DescriptorSetLayout {
    layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    fn create_bindings(
        binding_sizes: &[(vk::DescriptorType, u32)],
    ) -> Vec<vk::DescriptorSetLayoutBinding> {
        let mut layout_bindings =
            Vec::<vk::DescriptorSetLayoutBinding>::with_capacity(binding_sizes.len());

        for binding in binding_sizes {
            layout_bindings.push(vk::DescriptorSetLayoutBinding {
                binding: binding.0.as_raw() as u32,
                descriptor_type: binding.0,
                descriptor_count: binding.1,
                stage_flags: vk::ShaderStageFlags::ALL,
                ..Default::default()
            });
        }

        layout_bindings
    }

    pub fn new(
        device: &core::Device,
        binding_sizes: &[(vk::DescriptorType, u32)],
    ) -> DescriptorSetLayout {
        let bindings = DescriptorSetLayout::create_bindings(binding_sizes);

        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            flags: vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL,

            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),

            ..Default::default()
        };

        let layout = unsafe {
            device
                .get_ash_device()
                .create_descriptor_set_layout(&create_info, None)
        }
        .expect("Failed to create the descriptor set layout");

        DescriptorSetLayout { layout }
    }

    pub fn get_layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }
}
