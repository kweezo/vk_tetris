use std::ops::Deref;

use super::super::*;
use super::*;
use ash::vk;

#[derive(Clone)]
pub enum DescriptorInfo {
    Image(Vec<vk::DescriptorImageInfo>),
    Buffer(Vec<vk::DescriptorBufferInfo>),
    BufferView(Vec<vk::BufferView>),
}

pub struct DescriptorCreateInfo {
    pub descriptor_type: vk::DescriptorType,
    pub size: u32,
    pub binding: u32,
}

impl Deref for DescriptorInfo{
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        &self
    }
}

pub struct DescriptorSet {
    set: vk::DescriptorSet,
    layout: DescriptorSetLayout,
    pool: DescriptorPool,
}

impl DescriptorSet {
    pub fn new(
        device: &core::Device,
        descriptor_infos: &[DescriptorCreateInfo],
    ) -> DescriptorSet {
        let layout = DescriptorSetLayout::new(device, descriptor_infos);
        let pool = DescriptorPool::new(device, descriptor_infos);

        let allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: pool.get_pool(),
            descriptor_set_count: 1,
            p_set_layouts: &layout.get_layout(),
            ..Default::default()
        };

        let set = unsafe {
            device
                .get_ash_device()
                .allocate_descriptor_sets(&allocate_info)
        }
        .expect("Failed to allocate the descriptor set")[0];

        DescriptorSet { set, layout, pool }
    }

    pub fn create_write_set(
        &self,
        info: &DescriptorInfo,
        descriptor_type: vk::DescriptorType,
        array_element: u32,
        count: u32,
        binding: u32,
    ) -> vk::WriteDescriptorSet {
        let mut write_set = vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: self.set,

            dst_binding: binding,

            dst_array_element: array_element,
            descriptor_count: count,
            descriptor_type,

            ..Default::default()
        };

        match info {
            DescriptorInfo::Buffer(buffer_info) => write_set.p_buffer_info = buffer_info.as_ptr(),
            DescriptorInfo::Image(image_info) => write_set.p_image_info = image_info.as_ptr(),
            DescriptorInfo::BufferView(buffer_view_info) => {
                write_set.p_texel_buffer_view = buffer_view_info.as_ptr()
            }
        }

        write_set
    }

    pub fn update(&self, device: &core::Device, write_sets: &[vk::WriteDescriptorSet]) {
        unsafe {
            device
                .get_ash_device()
                .update_descriptor_sets(write_sets, &[])
        };
    }

    pub fn get_layout(&self) -> vk::DescriptorSetLayout {
        self.layout.get_layout()
    }

    pub fn get_set(&self) -> vk::DescriptorSet {
        self.set
    }
}
