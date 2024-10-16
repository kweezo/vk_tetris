use crate::*;

pub struct GL_Renderer{
}

impl GL_Renderer{
    fn init_opengl(window: &mut Window){
        gl::load_with(|s| window.get_window_mut().get_proc_address(s));
    }
}

impl renderer::Renderer for GL_Renderer{
    fn new(window: &mut Window) -> Self{
        GL_Renderer::init_opengl(window);

        Self{}
    }
}