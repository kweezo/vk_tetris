use ash::vk;
use super::*;
use super::super::*;

pub struct DescriptorSet{
    set: vk::DescriptorSet,
    layout: DescriptorSetLayout,
    pool: DescriptorPool
}

impl DescriptorSet{
    pub fn new(device: &core::Device, binding_sizes: &[(vk::DescriptorType, u32)]) -> DescriptorSet{
        let layout = DescriptorSetLayout::new(device, binding_sizes);
        let pool = DescriptorPool::new(device, binding_sizes);

        let allocate_info = vk::DescriptorSetAllocateInfo{
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: pool.get_pool(),
            descriptor_set_count: 1,
            p_set_layouts: &layout.get_layout(),
            ..Default::default()
        };

        let set = unsafe{device.get_ash_device().allocate_descriptor_sets(&allocate_info)}.expect("Failed to allocate the descriptor set")[0];

        DescriptorSet { set: set, layout: layout, pool: pool }
    }

    pub fn get_layout(&self) -> vk::DescriptorSetLayout{
        self.layout.get_layout()
    }

    pub fn get_set(&self) -> vk::DescriptorSet{
        self.set
    }
}