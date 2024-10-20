mod window;
use window::*;

mod vulkan;

fn main() {
    let mut window = Window::new(1280, 720, "le title");

    let core =vulkan::Core::new();

    while !window.get_window_handle().should_close(){
        window.get_glfw_context_mut().poll_events();
    }
}
