use crate::*;
use image::{DynamicImage, ImageReader, RgbImage};
use gl::types::*;


impl From<image::ImageError> for ImageError{
    fn from(err: image::ImageError) -> ImageError{
        ImageError::ImageLoad
    }
}

//impl From<image::ImageResult<DynamicImage>> for ImageError{
//    fn from(err: image::ImageError) -> ImageError{
//        ImageError::ImageLoad
//    }
//}


pub struct GLTexture{
    texture_handle: GLuint
}

impl GLTexture{
    #[inline(always)]
    fn open_image(path: &str) -> Result<image::DynamicImage, image::ImageError>{
        Ok(ImageReader::open(path)?.decode().unwrap())
    }

    fn gen_texture(image: &image::DynamicImage) -> GLuint{
        let mut tex: GLuint = 0;

        unsafe{
            gl::GenTextures(1 as GLsizei, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
        }
        0

    }
}


impl Texture for GLTexture{


    fn new(path: &str) -> Result<GLTexture, image::ImageError>{
        let image = GLTexture::open_image(path)?;
        let tex = GLTexture::gen_texture(&image);

        Ok(GLTexture{texture_handle: 0})
    }
}