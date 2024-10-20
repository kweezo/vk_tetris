

pub struct Window{
    glfw_context: glfw::Glfw,
    window_handle: glfw::PWindow,
}

impl Window{
    pub fn new(width: u32, height: u32, title: &str) -> Window{
        let mut context = glfw::init_no_callbacks().expect("Failed to init GLFW");
        let (window, _events) = context.create_window(width, height, title, glfw::WindowMode::Windowed).expect("Failed to create a GLFW window");

        Window{glfw_context: context, window_handle: window}
    }

    pub fn get_window_handle(&self) -> &glfw::PWindow{
        &self.window_handle
    }

    pub fn get_window_handle_mut(&mut self) -> &mut glfw::PWindow{
        &mut self.window_handle
    }

    pub fn get_glfw_context(&self) -> &glfw::Glfw{
        &self.glfw_context
    }

    pub fn get_glfw_context_mut(&mut self) -> &mut glfw::Glfw{
        &mut self.glfw_context
    }
}