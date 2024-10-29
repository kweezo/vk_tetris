use ash::{vk::{self}, Entry};
use std::{ffi::CStr, fs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config{
    pub required_extensions: Vec<String>,
    pub optional_extensions: Vec<String>
}

pub struct Instance{
    vk_instance: ash::Instance
}

impl Instance{

    fn load_config(path: &str) -> Config{
        let contents = fs::read_to_string(path)
        .expect("Failed to read the instance config file");
    
        serde_json::from_str(&contents).expect("Could not parse instance JSON config")
    }

    pub fn new(entry: &Entry) -> Instance{

        let conf = Instance::load_config("conf/instance.json");

        let app_name: &CStr;
        unsafe{app_name = std::ffi::CStr::from_bytes_with_nul_unchecked(b"tetris\0");}

        let app_info = vk::ApplicationInfo::default()
            .api_version( vk::API_VERSION_1_3)
            .application_version( vk::make_version(0, 0, 1))
            .engine_name(app_name)
            .application_name(app_name);
        
        let create_info : vk::InstanceCreateInfo =  vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&conf.required_extensions.as_ptr() as *[i8]);

        let instance: ash::Instance;
    
        unsafe {
            instance = entry.create_instance(&create_info, None).expect("Failed to create a vkInstance");
        }

        Instance{vk_instance: instance}
    }

    pub fn get_vk_instance(&self) -> &ash::Instance{
        &self.vk_instance
    }
}