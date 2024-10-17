
pub enum ImageError{
    ImageLoad,
}

pub trait Texture {
   fn new(path: &str) -> Result<Self, image::ImageError> where Self: Sized;
}