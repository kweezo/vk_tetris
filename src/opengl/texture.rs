use crate::*;
use image::{DynamicImage, ImageError, ImageReader, Rgb, RgbImage};
use gl::types::*;


pub struct GLTexture{
    texture_handle: GLuint
}

impl GLTexture{
    #[inline(always)]
    fn open_image(path: &str) -> Result<image::DynamicImage, image::ImageError>{
        Ok(ImageReader::open(path)?.decode()?)
    }

    fn gen_texture(image: &image::DynamicImage) -> Result<GLuint, TextureError>{
        let mut tex: GLuint = 0;

        unsafe{ // me when c+
            gl::GenTextures(1 as GLsizei, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

            use image::ColorType::*;

            let color_channel =  match image.color(){
                L8 => gl::RED,
                La8 => gl::RG,
                Rgb8 => gl::RGB,
                Rgba8 => gl::RGBA,
                L16 => gl::R16,
                La16 => gl::RG16,
                Rgb16 => gl::RGB16,
                Rgba16 => gl::RGBA16,
                _ => gl::NONE
            };

            if color_channel == gl::NONE{
                return Err(TextureError::InvalidColorChannel);
            }

        Ok(tex)
    }
}


impl Texture for GLTexture{


    fn new(path: &str) -> Result<GLTexture, TextureError>{
        let image = match GLTexture::open_image(path){
            Ok(res) => res,
            Err(err) => {
                match err{
                    ImageError::IoError(e) => return Err(TextureError::IoError(e)),
                    _ => return Err(TextureError::Miscellaneous)
                }
            }
        };

        let tex = GLTexture::gen_texture(&image)?;

        Ok(GLTexture{texture_handle: 0})
    }
}