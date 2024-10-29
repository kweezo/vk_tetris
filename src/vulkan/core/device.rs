use ash::vk::{self, DeviceQueueCreateInfo};


pub struct Device{
    device: ash::Device,
    physical_device: vk::PhysicalDevice
}

impl Device{
    fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice{
        let devices = unsafe{instance.enumerate_physical_devices()}
         .expect("Failed to enumerate the list of physical devices");

        let mut best_device: Option<vk::PhysicalDevice> = None; 

        for device in devices.iter(){
            let properties = unsafe{instance.get_physical_device_properties(*device)};

            if vk::PhysicalDeviceType::DISCRETE_GPU == properties.device_type{
                best_device = Some(*device);
            }
        }

        return match best_device{
            None => devices[0],
            Some(device) => device
        };
    }

    fn get_queue_family_indices(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Vec<(usize, vk::QueueFlags, u32)>{
        let properties = unsafe{instance.get_physical_device_queue_family_properties(physical_device)};

        let mut indices: Vec<(usize, vk::QueueFlags, u32)> = Vec::new();

        for i in 0..properties.len(){
            if properties[i].queue_flags & vk::QueueFlags::GRAPHICS == vk::QueueFlags::GRAPHICS || properties[i].queue_flags & vk::QueueFlags::TRANSFER == vk::QueueFlags::TRANSFER{
                indices.push((i, properties[i].queue_flags, properties[i].queue_count));
            }
        }

        indices
    }

    fn create_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, queue_family_indices: &Vec<(usize, vk::QueueFlags, u32)>) -> ash::Device{
        let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        for (index, _, count) in queue_family_indices.iter(){

            let priorities: Vec<f32> = vec![1f32; *count as usize];

            queue_create_infos.push(
                DeviceQueueCreateInfo{
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    queue_family_index: *index as u32,
                    queue_count: *count, 
                    p_queue_priorities: priorities.as_ptr(),
                    ..Default::default()
                }
            );  
        }

        let create_info = vk::DeviceCreateInfo{
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),    
            ..Default::default()
        };


        unsafe{instance.create_device(physical_device, &create_info, None)}.expect("Failed to create the Vulkan device")
    }

    pub fn new(instance: &ash::Instance) -> Device{
        let physical_device = Device::pick_physical_device(instance);
        let queue_family_indices = Device::get_queue_family_indices(instance, physical_device);
        let device = Device::create_device(instance, physical_device, &queue_family_indices);

        Device{device: device, physical_device: physical_device}
    }
}