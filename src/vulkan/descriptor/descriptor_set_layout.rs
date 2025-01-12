use std::os::raw::c_void;

use super::{*, super::*};
use ash::vk;

pub struct DescriptorSetLayout {
    layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    fn create_bindings(
        binding_sizes: &[descriptor_set::DescriptorCreateInfo],
    ) -> Vec<vk::DescriptorSetLayoutBinding> {
        let mut layout_bindings =
            Vec::<vk::DescriptorSetLayoutBinding>::with_capacity(binding_sizes.len());

        for descriptor_info in binding_sizes {
            layout_bindings.push(vk::DescriptorSetLayoutBinding {
                binding: descriptor_info.binding,
                descriptor_type: descriptor_info.descriptor_type,
                descriptor_count: descriptor_info.size,
                stage_flags: vk::ShaderStageFlags::ALL,
                ..Default::default()
            });
        }

        layout_bindings
    }

    pub fn new(
        device: &core::Device,
        descriptor_infos: &[descriptor_set::DescriptorCreateInfo],
    ) -> DescriptorSetLayout {
        let bindings = DescriptorSetLayout::create_bindings(descriptor_infos);

        let mut binding_flags =
            vec![vk::DescriptorBindingFlags::PARTIALLY_BOUND; descriptor_infos.len()];

        for i in 0..binding_flags.len() {
            if bindings[i].descriptor_type == vk::DescriptorType::INPUT_ATTACHMENT
                || bindings[i].descriptor_type == vk::DescriptorType::STORAGE_BUFFER_DYNAMIC
                || bindings[i].descriptor_type == vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
            {
                continue; //  "Validation Error: [ VUID-VkDescriptorSetLayoutBindingFlagsCreateInfo-None-03011 ] | MessageID = 0xe208328a | vkCreateDescriptorSetLayout(): pCreateInfo->pNext<VkDescriptorSetLayoutBindingFlagsCreateInfo>.pBindingFlags[0] includes VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT but pBindings[0].descriptorType is VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC. The Vulkan spec states: All bindings with descriptor type VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT, VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC, or VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC must not use VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT (https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#VUID-VkDescriptorSetLayoutBindingFlagsCreateInfo-None-03011)"
            }

            binding_flags[i] |= vk::DescriptorBindingFlags::UPDATE_AFTER_BIND;
        }

        let binding_flags_create_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO,
            binding_count: bindings.len() as u32,
            p_binding_flags: binding_flags.as_ptr(),
            ..Default::default()
        };

        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            flags: vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL,

            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),

            p_next: &binding_flags_create_info as *const _ as *const c_void,

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
