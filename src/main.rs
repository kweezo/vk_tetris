#![allow(dead_code)]

mod window;
use window::*;

mod vulkan;

fn main() {
    let mut window = Window::new(1280, 720, "le title");

    let core = vulkan::Core::new(&window);
    let _shader = vulkan::Shader::new(core.get_device(), String::from("shaders/bin/triangle_vert.spv"), String::from("shaders/bin/triangle_frag.spv"));

    while !window.get_window_handle().should_close(){
        window.get_glfw_context_mut().poll_events();
    }
}
