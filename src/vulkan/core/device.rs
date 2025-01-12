use ash::vk::{self, DeviceQueueCreateInfo, MAX_EXTENSION_NAME_SIZE};
use serde::Deserialize;
use std::{
    ffi::CString,
    fs,
    os::raw::c_void,
    sync::{Arc, RwLock},
};

#[derive(Deserialize, Debug)]
struct Config {
    pub required_extensions: Vec<String>,
    pub optional_extensions: Vec<String>,
}

pub struct Device {
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
    queues: Vec<ash::vk::Queue>,
    curr_queue_index: RwLock<u32>,

    allocator: Arc<RwLock<Arc<vk_mem::Allocator>>>,
}

impl Device {
    fn load_config(path: &str) -> Config {
        let contents = fs::read_to_string(path).expect("Failed to read the instance config file");

        serde_json::from_str(&contents).expect("Could not parse instance JSON config")
    }

    fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
        let devices = unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to enumerate the list of physical devices");

        let mut best_device: Option<vk::PhysicalDevice> = None;

        for device in devices.iter() {
            let properties = unsafe { instance.get_physical_device_properties(*device) };

            if vk::PhysicalDeviceType::DISCRETE_GPU == properties.device_type {
                best_device = Some(*device);
            }
        }

        match best_device {
            None => devices[0],
            Some(device) => device,
        }
    }

    fn get_queue_family_index_internal(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> (u32, u32) {
        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        for i in properties.iter().enumerate() {
            if i.1.queue_flags & vk::QueueFlags::GRAPHICS == vk::QueueFlags::GRAPHICS {
                return (i.0 as u32, i.1.queue_count);
            }
        }

        panic!("No graphics queue found, everything is fucked; unrecoverable");
    }

    fn create_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
        queue_count: u32,
        extensions: &[*const i8],
    ) -> ash::Device {
        let queue_priorities = vec![1f32; queue_count as usize];

        let create_info = DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            queue_family_index,
            queue_count,
            p_queue_priorities: queue_priorities.as_ptr(),
            ..Default::default()
        };

        let mut descriptor_indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures {
            s_type: vk::StructureType::PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES,

            runtime_descriptor_array: true as u32,

            descriptor_binding_partially_bound: true as u32,
            descriptor_binding_variable_descriptor_count: true as u32,

            shader_storage_buffer_array_non_uniform_indexing: true as u32,
            shader_sampled_image_array_non_uniform_indexing: true as u32,
            shader_storage_texel_buffer_array_non_uniform_indexing: true as u32,

            descriptor_binding_storage_buffer_update_after_bind: true as u32,
            descriptor_binding_sampled_image_update_after_bind: true as u32,
            descriptor_binding_storage_image_update_after_bind: true as u32,
            descriptor_binding_uniform_buffer_update_after_bind: true as u32,

            ..Default::default()
        };

        let device_features = vk::PhysicalDeviceFeatures2 {
            s_type: vk::StructureType::PHYSICAL_DEVICE_FEATURES_2,
            p_next: &mut descriptor_indexing_features as *mut _ as *mut c_void,
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: &device_features as *const _ as *const c_void,
            queue_create_info_count: 1,
            p_queue_create_infos: &create_info,
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        };

        unsafe { instance.create_device(physical_device, &create_info, None) }
            .expect("Failed to create the Vulkan device")
    }

    fn get_device_queues(
        device: &ash::Device,
        queue_family_index: u32,
        queue_count: u32,
    ) -> Vec<ash::vk::Queue> {
        let mut queues: Vec<ash::vk::Queue> = Vec::with_capacity(queue_count as usize);

        for i in 0..queue_count {
            queues.push(unsafe { device.get_device_queue(queue_family_index, i) });
        }

        queues
    }

    fn get_extensions(
        conf: &Config,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> (Vec<*const i8>, Vec<CString>) {
        let required_extensions_cstr: Vec<CString> =
            Device::get_supported_extensions(instance, physical_device, &conf.required_extensions);

        if required_extensions_cstr.len() != conf.required_extensions.len() {
            panic!("Error: not all required device extensions present");
        }

        let required_extensions_cstr_ptr: Vec<*const i8> = required_extensions_cstr
            .iter()
            .map(|s| s.as_ptr())
            .collect();

        let optional_extensions_cstr =
            Device::get_supported_extensions(instance, physical_device, &conf.optional_extensions);

        let optional_extensions_cstr_ptr: Vec<*const i8> = optional_extensions_cstr
            .iter()
            .map(|s| s.as_ptr())
            .collect();

        let mut extensions_cstr_ptr = required_extensions_cstr_ptr.clone();
        extensions_cstr_ptr.extend(optional_extensions_cstr_ptr.iter());

        let mut extensions_cstr = required_extensions_cstr;
        extensions_cstr.extend(optional_extensions_cstr);

        (extensions_cstr_ptr, extensions_cstr)
    }

    fn get_supported_extensions(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        extensions: &Vec<String>,
    ) -> Vec<CString> {
        let supported_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get enumerate instance extension properties")
        };

        let mut available_extensions: Vec<CString> = Vec::new();

        for supported_extension in &supported_extensions {
            for extension in extensions {
                let mut optional_extension_v: Vec<i8> =
                    extension.chars().map(|c| c as i8).collect();

                optional_extension_v.resize(MAX_EXTENSION_NAME_SIZE, 0);

                if supported_extension.extension_name == *optional_extension_v {
                    available_extensions.push(
                        CString::new(extension.as_str()).expect("Failed to create a new CString"),
                    );
                }
            }
        }

        available_extensions
    }

    fn create_allocator(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> vk_mem::Allocator {
        let create_info = vk_mem::AllocatorCreateInfo::new(instance, device, physical_device);

        unsafe {
            vk_mem::Allocator::new(create_info).expect("Failed to create a new VMA allocator")
        }
    }

    pub fn new(instance: &ash::Instance) -> Device {
        let conf = Device::load_config("conf/device.json");

        let physical_device = Device::pick_physical_device(instance);
        let (queue_family_index, queue_count) =
            Device::get_queue_family_index_internal(instance, physical_device);
        let (extensions_cstr_ptr, _extensions_cstr) =
            Device::get_extensions(&conf, instance, physical_device);

        let device = Device::create_device(
            instance,
            physical_device,
            queue_family_index,
            queue_count,
            &extensions_cstr_ptr,
        );

        let queues = Device::get_device_queues(&device, queue_family_index, queue_count);

        let allocator = Device::create_allocator(instance, physical_device, &device);

        Device {
            device,
            physical_device,
            queue_family_index,
            queues,
            curr_queue_index: RwLock::new(0),
            allocator: Arc::new(RwLock::new(Arc::new(allocator))),
        }
    }

    pub fn get_ash_device(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_vk_physical_device(&self) -> ash::vk::PhysicalDevice {
        self.physical_device
    }

    pub fn get_allocator(&self) -> Arc<vk_mem::Allocator> {
        self.allocator.read().unwrap().clone()
    }

    pub fn get_allocator_lock(&self) -> Arc<RwLock<Arc<vk_mem::Allocator>>> {
        self.allocator.clone()
    }

    pub fn get_queue_family_index(&self) -> u32 {
        self.queue_family_index
    }
    pub fn get_queue(&self) -> ash::vk::Queue {
        let queue = self.queues[*self.curr_queue_index.read().unwrap() as usize];
        let queue_index = *self.curr_queue_index.read().unwrap();
        *self.curr_queue_index.write().unwrap() = (queue_index + 1) % self.queues.len() as u32;
        queue
    }
}
