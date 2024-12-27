use ash::vk;

use super::core::*;

pub struct Fence {
    fence: vk::Fence,
}

impl Fence {
    pub fn new(device: &Device, signaled: bool) -> Fence {
        let create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: if signaled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            },
            ..Default::default()
        };

        let fence = unsafe { device.get_ash_device().create_fence(&create_info, None) }
            .expect("Failed to create a vulkan fence");

        Fence { fence }
    }

    pub fn get_fence(&self) -> vk::Fence {
        self.fence
    }
}
