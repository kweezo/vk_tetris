use crate::{types::Rect, *};
use stb_truetype_rust::*;
use std::{fs, pin::Pin, ptr::copy_nonoverlapping};
use descriptor::{DescriptorInfo, DescriptorSet};
use ash::vk;


pub struct TextRenderer{
    font_atlas_tex: Texture,
    paddings: Vec<f32>,
    heights: Vec<f32>,
    char_count: u32,
    starting_offset: u32,
    chars_per_row: u32,
    row_count: u32
}

pub struct RenderInfo {
    pub char_count: u32,
    pub rect: Rect
}

impl<'a> TextRenderer{
    pub fn new(core: &Core, device: &Device, command_pool: &CommandPool) -> TextRenderer{

        let char_count = 93;
        let starting_offset = 33;

        let (font_atlas_tex, paddings, heights, chars_per_row, row_count)= TextRenderer::load_font_atlas(core, device, command_pool, char_count, starting_offset);

        TextRenderer { font_atlas_tex, char_count, starting_offset, paddings, heights, chars_per_row, row_count}
    }

    fn load_font_atlas(core: &Core, device: &Device, command_pool: &CommandPool, char_count: u32, starting_offset: u32) -> (Texture, Vec<f32>, Vec<f32>, u32, u32){
        let dat: Vec<u8> = fs::read("font.otf").expect("Failed to read font.ttf");

        let mut font_info = stbtt_fontinfo::default();
        unsafe { assert!(stbtt_InitFont(&mut font_info, dat.as_ptr(), 0) != 0, "Failed to parse the font") }; 
            
        let (data, paddings, heights, width, height, chars_per_row, row_count) = TextRenderer::create_font_bitmap(core, &mut font_info, 1f32,
             starting_offset, char_count);

        let tex = TextRenderer::create_font_texture(
            device,
            command_pool,
            &data,
            width,
            height
        );

        (tex, paddings, heights, chars_per_row, row_count)
    }

    fn create_font_bitmap(core: &Core, font_info: &mut stbtt_fontinfo, scale: f32, starting_offset: u32, char_count: u32) -> (Vec<u8>, Vec<f32>, Vec<f32>, u32, u32, u32, u32) {
        let (mut max_width, mut max_height) = (0i32, 0i32);
        //set max image width to something divisibe by max_width so padding isnt a bitchg

        let format_properties =
         unsafe {
            core.get_instance().get_ash_instance().get_physical_device_image_format_properties(core.get_device().get_vk_physical_device(),
         vk::Format::R8_SRGB,
            vk::ImageType::TYPE_2D,
         vk::ImageTiling::OPTIMAL,
          vk::ImageUsageFlags::SAMPLED,
          vk::ImageCreateFlags::empty())
         }.expect("Failed to get physical device image format properties");

        //get the max spacing for characters
        for i in 33..starting_offset + char_count{
            let (mut x0, mut y0, mut x1, mut y1) = (0i32, 032, 0i32, 0i32);

            unsafe {stbtt_GetCodepointBitmapBox(font_info, i as i32, scale, scale, &mut x0, &mut y0, &mut x1, &mut y1); }

            let width = x1 - x0;
            let height = y1 - y0;

            max_width = max_width.max(width);
            max_height = max_height.max(height);

        }

        let chars_per_row = (format_properties.max_extent.width as f32 / max_width as f32).floor() as u32;
        let image_width = chars_per_row * max_width as u32;

        let row_count = ((char_count * max_width as u32) as f32 / image_width as f32).ceil() as u32;
        let mut current_row = 0u32;

        let mut curr_chars_per_row = 0u32;


//        let mut pixels = vec![0u8; (max_width * max_height * char_count as i32 * (char_count as f32 / max_chars_per_row as f32).ceil() as i32 + ((1.0 - ((char_count * max_width as u32) as f32 / max_image_width as f32).fract()) * max_image_width as f32) as i32 * max_height) as usize];
        let mut pixels = vec![0u8; (image_width as i32 * (row_count + 1) as i32 * max_height as i32) as usize];
    
        let mut paddings = Vec::<f32>::with_capacity(char_count as usize);
        let mut heights = Vec::<f32>::with_capacity(char_count as usize);

        //splice the character bitmaps together
        for (i, c) in (starting_offset..char_count + starting_offset).enumerate(){
            let (mut width, mut height, mut xoff, mut yoff) = (0, 0, 0, 0);
            let bmp_ptr = unsafe { stbtt_GetCodepointBitmap(font_info, scale, scale,
                 c as i32, &mut width, &mut height, &mut xoff, &mut yoff) };

            assert!(bmp_ptr != std::ptr::null_mut(), "Failed to get the bitmap for char {c} (bmp_ptr is null)");


            let mut bmp = vec![0u8; (width*height) as usize];


            unsafe { copy_nonoverlapping(bmp_ptr, bmp.as_mut_ptr(), bmp.len()) };


            let offset = i * max_width as usize; 
            let padding = ((max_width-width) as f32 / 2f32).floor() as usize;

            heights.push(height as f32 / max_height as f32);

            for (i, y) in (max_height-height..max_height-1).enumerate() {

                let starting_offset = offset + (y * image_width as i32) as usize + padding +
                 image_width as usize * current_row as usize * max_height as usize;

                pixels.splice(starting_offset..starting_offset + width as usize,
                     bmp[(i as i32 * width) as usize..(i as i32 * width + width) as usize].iter().cloned());
            }

            paddings.push(padding as f32 / max_width as f32);

            curr_chars_per_row += 1;

            if curr_chars_per_row >= chars_per_row {
                curr_chars_per_row = 0;
                current_row += 1;
            }
        }

        pixels.shrink_to_fit();

        (pixels, paddings, heights, image_width, row_count * max_height as u32, chars_per_row, row_count)
    }

    fn create_font_texture(device: &Device, command_pool: &CommandPool, data: &Vec<u8>, width: u32, height: u32) -> Texture {
        let mut command_buffer = CommandBuffer::new(device, command_pool, false);
        command_buffer.begin(device, &vk::CommandBufferInheritanceInfo::default(), vk::CommandBufferUsageFlags::empty());

        let tex = Texture::new_raw_data(
            device,
            &mut command_buffer,
            data.as_slice(),
            width,
            height,
            vk::Format::R8_SRGB,
            false).expect("Failed to create the font atlas texture");

        command_buffer.end(device);



        let fence = Fence::new(device, false);

        CommandBuffer::submit(device, &[command_buffer.get_command_buffer()], &[], &[], fence.get_fence());

        unsafe{
            device.get_ash_device().wait_for_fences(&[fence.get_fence()], true, std::u64::MAX)
            .expect("Failed to waitd for the font atlas load fence");
        }

        command_buffer.cleanup(device);

        tex
    }

    pub fn get_data_for_str(&self, string: &str, rect: &Rect) -> (Vec<u8>, (u32, u32), u32) {
        let mut dat = Vec::<u8>::with_capacity(string.len());

        let mut curr_padding = 0f32;
        for (i, c) in string.chars().enumerate() {
            //let char_dat = self.char_data[c as usize - 32];

            if c as u32 == 32 {// space
                dat.push(255);

                dat.extend_from_slice(&[0u8; 3]);
                dat.extend_from_slice(&1u32.to_ne_bytes());

                curr_padding += 1f32;
           
                continue;
            } 

            dat.push(c as u8 - self.starting_offset as u8);


            curr_padding += self.paddings[c as usize - self.starting_offset as usize];
            if i != 0 {
                if string.as_bytes()[i-1] != b' ' {
                    curr_padding += self.paddings[string.as_bytes()[i-1] as usize - self.starting_offset as usize];
                }
            } 
           
            dat.extend_from_slice(&[0u8; 3]);
            dat.extend_from_slice(&(curr_padding).to_ne_bytes());


        }

        let (scale, y_offset) = self.calculate_adjusted_scale(string, (rect.width, rect.height));

        (dat, scale, y_offset)
    }

    pub fn prepare_text_renderer(&self, device: &Device, command_buffer: &CommandBuffer, vertex_buffer: &Buffer, index_buffer: &Buffer, render_pass: &RenderPass, subpass_index: u32) {
        unsafe {
            device.get_ash_device().cmd_next_subpass(
                command_buffer.get_command_buffer(),
                vk::SubpassContents::INLINE,
            );

            let push_constants = 2u32.to_ne_bytes();

            device.get_ash_device().cmd_push_constants(command_buffer.get_command_buffer(), render_pass.get_layout(), vk::ShaderStageFlags::ALL,
             0, &push_constants);

            device.get_ash_device().cmd_bind_pipeline(
                command_buffer.get_command_buffer(),
                vk::PipelineBindPoint::GRAPHICS,
                render_pass.get_pipeline(subpass_index as usize),
            );

            device.get_ash_device().cmd_bind_vertex_buffers(command_buffer.get_command_buffer(), 0,
             &[vertex_buffer.get_buffer()], &[0]);

            device.get_ash_device().cmd_bind_index_buffer(command_buffer.get_command_buffer(), index_buffer.get_buffer(), 0, vk::IndexType::UINT16);
        }
    }

    pub fn render_text(&self, device: &Device, command_buffer: &CommandBuffer, render_pass: &RenderPass, data_buffer: &Buffer, info: RenderInfo) {

        let push_constants = [self.char_count.to_ne_bytes().as_slice(), info.rect.to_ne_bytes().as_slice(),
         info.char_count.to_ne_bytes().as_slice(), self.chars_per_row.to_ne_bytes().as_slice(), self.row_count.to_ne_bytes().as_slice()].concat();

        unsafe{

            device.get_ash_device().cmd_push_constants(command_buffer.get_command_buffer(), render_pass.get_layout(), vk::ShaderStageFlags::ALL,
             4, &push_constants);

            device.get_ash_device().cmd_bind_vertex_buffers(command_buffer.get_command_buffer(), 1,
             &[data_buffer.get_buffer()], &[0]);

            device.get_ash_device().cmd_draw_indexed(command_buffer.get_command_buffer(), 6, info.char_count,
             0, 0, 0);
        }
    }

    fn calculate_adjusted_scale(&self, string: &str, target_size: (u32, u32)) -> ((u32, u32), u32) {
        let mut max_height = 0f32;

        let mut paddings_sum = 0.0f32;
        let instance_count = string.chars().count() as f32;

        for c in string.chars() {
            paddings_sum += self.paddings[c as usize - self.starting_offset as usize];
            max_height = max_height.max(self.heights[c as usize - self.starting_offset as usize])
        }

        let denominator = 1.0 - (2.0 * paddings_sum) / instance_count;

        let adjusted_x = (target_size.0 as f32 / denominator).round() as u32;
        let adjusted_y = (target_size.1 as f32 / max_height) as u32;

        ((adjusted_x, adjusted_y), ((adjusted_y as f32) * (1.0f32 - max_height as f32)) as u32)
    }

      pub fn get_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet<'a>; 1], Vec<Pin<Box<DescriptorInfo>>>) {

        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.font_atlas_tex.get_image_view(),
            sampler: self.font_atlas_tex.get_sampler(),
        };




        let infos = vec![
            Pin::new(
                Box::new(
            DescriptorInfo::Image(vec![image_info])
        ))];

        let image_descriptor_write = set.create_write_set(
            &infos[0],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            2, 
            1,
            1
        );

        ([image_descriptor_write], infos)
    }

    pub fn destroy(&mut self, device: &Device) {
        self.font_atlas_tex.destroy(device);
    }
}