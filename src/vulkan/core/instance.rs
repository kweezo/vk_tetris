use ash::{
    vk::{self, MAX_EXTENSION_NAME_SIZE},
    Entry,
};
use serde::Deserialize;
use std::{ffi::CStr, ffi::CString, fs};

use crate::Window;

#[derive(Deserialize, Debug)]
struct Config {
    pub required_extensions: Vec<String>,
    pub optional_extensions: Vec<String>,
    pub validation_layers: Vec<String>,
    pub enable_layers: bool,
}

pub struct Instance {
    vk_instance: ash::Instance,
}

impl Instance {
    fn load_config(path: &str) -> Config {
        let contents = fs::read_to_string(path).expect("Failed to read the instance config file");

        serde_json::from_str(&contents).expect("Could not parse instance JSON config")
    }

    //Recieves a vector of extensions and returns the ones that are supported
    fn get_supported_extensions(entry: &Entry, extensions: &Vec<String>) -> Vec<CString> {
        let supported_extensions = unsafe {
            entry
                .enumerate_instance_extension_properties(None)
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

    fn get_supported_layers(entry: &Entry, layers: &Vec<String>) -> (Vec<CString>, Vec<String>) {
        let available_layers = unsafe {
            entry
                .enumerate_instance_layer_properties()
                .expect("Failed to enumreate supported instance layers")
        };

        let mut supported_layers: Vec<CString> = Vec::new();
        let mut unsupported_layers: Vec<String> = Vec::new();

        for layer in layers {
            let mut found = false;

            for supported_layer in &available_layers {
                let mut layer_v: Vec<i8> = layer.chars().map(|c| c as i8).collect();

                layer_v.resize(MAX_EXTENSION_NAME_SIZE, 0);

                if supported_layer.layer_name == *layer_v {
                    supported_layers.push(
                        CString::new(layer.as_str()).expect("Failed to create a new CString"),
                    );
                    found = true;
                }
            }

            if !found {
                unsupported_layers.push(layer.clone());
            }
        }

        (supported_layers, unsupported_layers)
    }

    fn get_extensions(
        conf: &Config,
        entry: &Entry,
        window: &Window,
    ) -> (Vec<*const i8>, Vec<CString>) {
        let mut glfw_extensions = window
            .get_glfw_context()
            .get_required_instance_extensions()
            .expect("Failed to get list of the required GLFW instance extensions");
        glfw_extensions.extend(conf.required_extensions.iter().cloned());

        let required_extensions_cstr: Vec<CString> =
            Instance::get_supported_extensions(entry, &glfw_extensions);

        if required_extensions_cstr.len() != conf.required_extensions.len() + glfw_extensions.len()
        {
            panic!("Error: not all required instance extensions present");
        }

        let required_extensions_cstr_ptr: Vec<*const i8> = required_extensions_cstr
            .iter()
            .map(|s| s.as_ptr())
            .collect();

        let optional_extensions_cstr =
            Instance::get_supported_extensions(&entry, &conf.optional_extensions);

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

    pub fn new(entry: &Entry, window: &Window) -> Instance {
        let conf = Instance::load_config("conf/instance.json");

        let (supported_layers_cstr, unsupported_layers) = if conf.enable_layers {
            Instance::get_supported_layers(&entry, &conf.validation_layers)
        } else {
            (Vec::new(), Vec::new())
        };

        if conf.enable_layers && !unsupported_layers.is_empty() {
            println!("Warning: some instance validation layers are unsupported:");
            for layer in &unsupported_layers {
                println!("\t{layer}");
            }
        }

        let (extensions_cstr_ptr, _extensions_cstr) =
            Instance::get_extensions(&conf, entry, window);

        let supported_layers_cstr_ptr: Vec<*const i8> =
            supported_layers_cstr.iter().map(|s| s.as_ptr()).collect();

        let app_name: &CStr = unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"tetris\0") };

        #[allow(deprecated)]
        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_3)
            .application_version(vk::make_version(0, 0, 1))
            .engine_name(app_name)
            .application_name(app_name);

        let create_info: vk::InstanceCreateInfo = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions_cstr_ptr)
            .enabled_layer_names(&supported_layers_cstr_ptr);

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create a vkInstance")
        };

        Instance {
            vk_instance: instance,
        }
    }

    pub fn get_ash_instance(&self) -> &ash::Instance {
        &self.vk_instance
    }
}