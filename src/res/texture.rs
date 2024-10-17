

pub enum TextureError{
    IoError(std::io::Error),
    InvalidColorChannel,
    Miscellaneous
}

pub trait Texture {
   fn new(path: &str) -> Result<Self, TextureError> where Self: Sized;
}