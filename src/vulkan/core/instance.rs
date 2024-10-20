use ash::{vk::{self}, Entry};
use std::ffi::CStr;

pub struct Instance{
    vk_instance: ash::Instance
}

impl Instance{
    pub fn new(entry: &Entry) -> Instance{
    
        let app_name: &CStr;
        unsafe{app_name = std::ffi::CStr::from_bytes_with_nul_unchecked(b"tetris\0");}

        let app_info =  vk::ApplicationInfo::default()
            .api_version( vk::API_VERSION_1_3)
            .application_version( vk::make_version(0, 0, 1))
            .engine_name(app_name)
            .application_name(app_name);
        
        let create_info : vk::InstanceCreateInfo =  vk::InstanceCreateInfo::default()
        .application_info(&app_info);

        let instance: ash::Instance;
    
        unsafe {
            instance = entry.create_instance(&create_info, None).expect("Failed to create a vkInstance");
        }

        Instance{vk_instance: instance}
    }
}