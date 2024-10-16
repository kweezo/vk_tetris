use glfw::PWindow;


#[derive(Debug)]
pub enum WindowError{
    InitError(glfw::InitError),
}

impl From<glfw::InitError> for WindowError{
    fn from(err: glfw::InitError) -> WindowError{
        WindowError::InitError(err)
    }
}

/// A GLFW window wrapper
/// provides basic functionality of creating a window and polling for events
pub struct Window{
    glfw: glfw::Glfw,
    window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,

    width: u32,
    height: u32,
    title: String
}

impl Window{
    /// Creates a new instance of a GLFW window
    /// 
    /// # Arguments
    /// 
    /// `width`: specifices window width in pixels\
    /// `height`: specifices window height in pixels\
    /// `title`: specifices the window title\
    /// `fulscreen`: specifies whether the window is fullscreen\
    /// 
    /// # Returns
    /// 
    /// `Result<Window, WindowError>`
    /// 
    /// # Example
    /// 
    /// ```
    /// 
    /// let window = Window::new(1920, 1080, "This is a window!", false).unwrap();
    /// 
    /// ```
    pub fn new(width: u32, height: u32, title: String, fullscreen: bool) -> Result<Window, WindowError>{
        let mut glfw = glfw::init_no_callbacks()?;

        
        let (mut window, events) = glfw.create_window(width, height, title.as_str(), glfw::WindowMode::Windowed)
        .expect("Failed to create the window");

        Ok(Window{glfw: glfw, window: window, events: events, width: width, height: height, title: title.clone()})
    }

    /// Polls the window events like keypresses or mouse clicks - **needs to be called every frame**
    /// 
    /// # Example
    /// 
    /// ```
    /// window.poll_events()
    /// ```
    pub fn poll_events(&mut self){
       self.glfw.poll_events(); 

       //TODO
    }

    /// Returns whether the window *should* close
    /// 
    /// # Returns
    ///
    /// `bool` - dictates whether the window should close
    /// 
    /// # Example
    /// 
    /// ```
    /// while !window.should_close(){
    /// ```
    pub fn should_close(&self) -> bool{
        self.window.should_close()
    }

    /// Returns the GLFW window handle of type `PWindow` 
    pub fn get_window(&self) -> &PWindow{
        &self.window
    }

    /// Returns the mutable GLFW window handle of type `PWindow` 
    pub fn get_window_mut(&mut self) -> &mut PWindow{
        &mut self.window
    }

    /// Returns the GLFW context handle of type `Glfw` 
    pub fn get_glfw(&self) -> &glfw::Glfw{
        &self.glfw
    }
    
    /// Returns the mutable GLFW context handle of type `Glfw` 
    pub fn get_glfw_mut(&mut self) -> &mut glfw::Glfw{
        &mut self.glfw
    }
}