use ash::vk;
use super::*;
use super::super::*;


pub enum DescriptorInfo{
    Image(Vec<vk::DescriptorImageInfo>),
    Buffer(Vec<vk::DescriptorBufferInfo>),
    BufferView(Vec<vk::BufferView>),
}

pub struct DescriptorSet{
    set: vk::DescriptorSet,
    layout: DescriptorSetLayout,
    pool: DescriptorPool,

    descriptor_infos: Vec<DescriptorInfo>,
    write_sets: Vec<vk::WriteDescriptorSet>,
}

impl <'a>DescriptorSet{
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

        DescriptorSet { set: set, layout: layout, pool: pool, write_sets: Vec::new(), descriptor_infos: Vec::new()}
    }

    pub fn add_write(&mut self, info: DescriptorInfo, descriptor_type: vk::DescriptorType, array_element: u32, count: u32){
        let mut write_set = vk::WriteDescriptorSet{
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: self.set,
            
            dst_binding: descriptor_type.as_raw() as u32,

            dst_array_element: array_element,
            descriptor_count: count,
            descriptor_type: descriptor_type,

            ..Default::default()
        };

        match info{
            DescriptorInfo::Buffer(buffer_info) => write_set.p_buffer_info = buffer_info.as_ptr(),
            DescriptorInfo::Image(image_info) => write_set.p_image_info = image_info.as_ptr(),
            DescriptorInfo::BufferView(buffer_view_info) => write_set.p_texel_buffer_view = buffer_view_info.as_ptr()
        }

        self.write_sets.push(write_set); 
        self.descriptor_infos.push(info);
    }

    pub fn update(&mut self, device: &core::Device){
        unsafe{device.get_ash_device().update_descriptor_sets(&self.write_sets.as_slice(), &[])};

        self.write_sets.clear();
    }

    pub fn get_layout(&self) -> vk::DescriptorSetLayout{
        self.layout.get_layout()
    }

    pub fn get_set(&self) -> vk::DescriptorSet{
        self.set
    }
}