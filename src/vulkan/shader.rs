use super::core::*;
use ash::vk;

pub struct Shader {
    vert_module: vk::ShaderModule,
    frag_module: vk::ShaderModule,
}

impl Shader {
    fn read_binary(path: String) -> Vec<u8> {
        let bytes = std::fs::read(&path);
        let bytes_unwrapped: Vec<u8> = match bytes {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Failed to open file {} because {}", &path, err);
                return Vec::new();
            }
        };

        bytes_unwrapped
    }

    fn create_shader_module(
        device: &Device,
        vert_path: String,
        frag_path: String,
    ) -> (vk::ShaderModule, vk::ShaderModule) {
        let vert_src = Shader::read_binary(vert_path);
        let frag_src = Shader::read_binary(frag_path);

        let mut create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: vert_src.len(),
            p_code: vert_src.as_ptr() as *mut u32,
            ..Default::default()
        };

        let vert_module = unsafe {
            device
                .get_ash_device()
                .create_shader_module(&create_info, None)
        }
        .expect("Failed to create the vertex shader module");

        create_info.code_size = frag_src.len();
        create_info.p_code = frag_src.as_ptr() as *mut u32;

        let frag_module = unsafe {
            device
                .get_ash_device()
                .create_shader_module(&create_info, None)
        }
        .expect("Failed to create the vertex shader module");

        (vert_module, frag_module)
    }

    pub fn new(device: &Device, vert_path: String, frag_path: String) -> Shader {
        let (vert_module, frag_module) = Shader::create_shader_module(device, vert_path, frag_path);

        Shader {
            vert_module,
            frag_module,
        }
    }

    pub fn get_pipeline_stage_shader_info(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let vert_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::VERTEX,
            module: self.vert_module,
            p_name: b"main\0".as_ptr() as *const i8,
            ..Default::default()
        };

        let frag_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: self.frag_module,
            p_name: b"main\0".as_ptr() as *const i8,
            ..Default::default()
        };

        vec![vert_info, frag_info]
    }
}
