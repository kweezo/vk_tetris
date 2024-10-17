use crate::*;

pub enum Graphics_API{
    OPENGL,
    VULKAN   
}

///
/// A generic renderer type used for multiple graphics APIs.
/// To create a new renderer instance use `create_renderer`
/// 
pub trait Renderer{
    fn new(window: &mut Window) -> Self;

    ///Sets the clear color of the screen
    fn set_clear_color(&mut self, color: glm::Vec3);

    ///This method is used to update the frame; *needs to be called once every frame*
    /// 
    ///  # Params
    /// 
    /// 'window': specifices the window that will be rendered too\ 
    /// 
    ///  # Example
    /// 
    /// ```
    /// 
    /// render(window);
    /// 
    /// ```
    fn render(&self, window: &mut Window);


    fn create_texture(&self, path: &str) -> Result<impl Texture, TextureError>;
} 

///
/// Creates a new renderer instance
/// 
/// # Params
/// 
/// `api`: specifies the graphics api that will be used/
/// `window`: specifices the user created window that will be used/
/// 
///  # Returns 
/// 
/// `impl Renderer`
/// 
/// # Example
/// 
/// `let renderer = create_renderer(Graphics_API::OPENGL, window)`
/// 
pub fn create_renderer(_api: Graphics_API, window: &mut Window) -> impl Renderer{
    GLRenderer::new(window)
}