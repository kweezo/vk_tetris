use glfw::Context;

use crate::*;

pub struct GLRenderer{
    clear_color: glm::Vec3,
}

impl GLRenderer{
    fn init_opengl(window: &mut Window){
        gl::load_with(|s| window.get_window_mut().get_proc_address(s));


        let (window_width, window_height) = window.get_size();
        unsafe{ 
            gl::Viewport(0, 0, window_width as i32, window_height as i32); 
            gl::ClearColor(0f32, 0f32, 0f32, 1f32);
        }
    }
}

impl renderer::Renderer for GLRenderer{
    fn new(window: &mut Window) -> Self{
        GLRenderer::init_opengl(window);

        Self{clear_color: glm::Vector3::new(0f32, 0f32, 0f32)}
    }

    fn set_clear_color(&mut self, color: glm::Vec3){
        self.clear_color = color;
        unsafe{ gl::ClearColor(color.x, color.y, color.z, 1f32) }
    }

    fn render(&self, window: &mut Window){
        unsafe{ gl::Clear(gl::COLOR_BUFFER_BIT) }

        window.get_window_mut().swap_buffers();
    }
}