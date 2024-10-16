
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
pub fn create_renderer(api: Graphics_API, window: &mut Window) -> impl Renderer{
    GL_Renderer::new(window)
}