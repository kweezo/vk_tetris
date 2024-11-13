pub struct Window {
    glfw_context: glfw::Glfw,
    window_handle: glfw::PWindow,
    width: u32,
    height: u32,
}

impl Window {
    fn error_callback(err: glfw::Error, description: String) {
        eprintln!("GLFW error {:?}: {:?}", err, description);
    }

    pub fn new(width: u32, height: u32, title: &str) -> Window {
        let mut context = glfw::init(Window::error_callback).expect("");

        context.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        let (window, _events) = context
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create a GLFW window");

        Window {
            glfw_context: context,
            window_handle: window,
            width: width,
            height: height,
        }
    }

    pub fn get_window_handle(&self) -> &glfw::Window {
        &self.window_handle
    }

    pub fn get_window_handle_mut(&mut self) -> &mut glfw::Window {
        &mut self.window_handle
    }

    pub fn get_glfw_context(&self) -> &glfw::Glfw {
        &self.glfw_context
    }

    pub fn get_glfw_context_mut(&mut self) -> &mut glfw::Glfw {
        &mut self.glfw_context
    }

    pub fn get_extent(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
