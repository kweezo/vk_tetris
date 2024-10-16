use image::{ImageReader, RgbImage};
use gl::types::*;



pub struct Texture{
    texture_handle: GLuint
}

impl Texture{

    #[inline(always)]
    fn open_image(path: &str) -> Result<RgbImage, image::ImageError>{
        Ok(ImageReader::open(path)?.decode()?.to_rgb8())
    }

    fn gen_texture() -> GLuint{
        let mut tex: GLuint = 0;

        unsafe{
            gl::GenTextures(1 as GLsizei, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
        }

        tex
    }

    pub fn new(path: &str) -> Result<Texture, image::ImageError>{
        let image = Texture::open_image(path)?;

        Ok(Texture{texture_handle: 0})
    }
}