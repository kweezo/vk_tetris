use glfw::WindowEvent;

pub struct Window {
    glfw_context: glfw::Glfw,
    events: glfw::GlfwReceiver<(f64, WindowEvent)>,
    window_handle: glfw::PWindow,
    width: u32,
    height: u32,
}

static mut WIDTH: u32 = 0u32;
static mut HEIGHT: u32 = 0u32;

fn size_callback(_window: &mut glfw::Window, new_width: i32, new_height: i32) {

    if new_width == 0 || new_height == 0 {
        unsafe{
            WIDTH = 1;
            HEIGHT = 1;
        }
        return;
    }

    unsafe{
        WIDTH = new_width as u32;
        HEIGHT = new_height as u32;
    }
}

impl Window {

    fn error_callback(err: glfw::Error, description: String) {
        eprintln!("GLFW error {:?}: {:?}", err, description);
    }

    pub fn new(width: u32, height: u32, title: &str) -> Window {
        let mut context = glfw::init(Window::error_callback).expect("");

        context.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        let (mut window, events) = context
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create a GLFW window");

        window.set_key_polling(true);
        window.set_size_callback(size_callback);

        unsafe {
            WIDTH = width;
            HEIGHT = height;
        }

        Window {
            glfw_context: context,
            events,
            window_handle: window,
            width,
            height,
        }
    }

    pub fn update_size(&self) {
        let new_size = self.window_handle.get_size();

        if new_size.0 == 0 || new_size.1 == 0 {
            unsafe{
                WIDTH = 1;
                HEIGHT = 1;
            } 

            return;
        }

        unsafe{
            WIDTH = new_size.0 as u32;
            HEIGHT = new_size.0 as u32;
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

    pub fn get_events(&self) -> &glfw::GlfwReceiver<(f64, WindowEvent)> {
        &self.events
    }

    pub fn get_extent(&self) -> (u32, u32) {
        unsafe{
            (WIDTH, HEIGHT)
        }
    }
}
