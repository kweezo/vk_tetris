use crate::{*, core::Device};
use stb_truetype_rust::*;
use std::{fs, pin::Pin, ptr::copy_nonoverlapping, slice::from_raw_parts};
use descriptor::DescriptorInfo;


pub struct TextRenderer{
    font_atlas_tex: Texture,
    offset_table_buffer: Buffer,
    char_count: u32,
    starting_offset: u32,
}

pub struct RenderInfo {
    pub char_count: u32,
    pub scale: f32,
    pub pos: (u32, u32),
}

impl<'a> TextRenderer{
    pub fn new(device: &Device, command_pool: &CommandPool) -> TextRenderer{

        let char_count = 93;
        let starting_offset = 33;

        let (font_atlas_tex, offset_table_buffer)= TextRenderer::load_font_atlas(device, command_pool, char_count, starting_offset);

        TextRenderer { font_atlas_tex, char_count: char_count, starting_offset, offset_table_buffer}
    }

    fn load_font_atlas(device: &Device, command_pool: &CommandPool, char_count: u32, starting_offset: u32) -> (Texture, Buffer){
        let dat: Vec<u8> = fs::read("Super Cartoon.ttf").expect("Failed to read font.ttf");

        let mut font_info = stbtt_fontinfo::default();
        unsafe { assert!(stbtt_InitFont(&mut font_info, dat.as_ptr(), 0) != 0, "Failed to parse the font") }; 
            
        let (mut max_width, mut max_height) = (0i32, 0i32);

        let mut offset_table = Vec::<f32>::with_capacity(char_count as usize);

        //get the max spacing for characters
        for i in 33..starting_offset + char_count{
            let (mut x0, mut y0, mut x1, mut y1) = (0i32, 032, 0i32, 0i32);

            unsafe {stbtt_GetCodepointBitmapBox(&mut font_info, i as i32, 0.3f32, 0.3f32, &mut x0, &mut y0, &mut x1, &mut y1); }

            let width = x1 - x0;
            let height = y1 - y0;

            max_width = max_width.max(width);
            max_height = max_height.max(height);

        }

        let mut pixels = vec![0u8; (max_width * max_height * char_count as i32) as usize];

        //splice the character bitmaps together
        for (i, c) in (starting_offset..char_count + starting_offset).enumerate(){
            let (mut width, mut height, mut xoff, mut yoff) = (0, 0, 0, 0);
            let bmp_ptr = unsafe { stbtt_GetCodepointBitmap(&mut font_info, 0.3f32, 0.3f32,
                 c as i32, &mut width, &mut height, &mut xoff, &mut yoff) };

            assert!(bmp_ptr != std::ptr::null_mut(), "Failed to get the bitmap for char {c} (bmp_ptr is null)");


            let mut bmp = vec![0u8; (width*height) as usize];


            unsafe { copy_nonoverlapping(bmp_ptr, bmp.as_mut_ptr(), bmp.len()) };


            let offset = i * max_width as usize; 
            let padding = ((max_width-width) as f32 / 2f32).floor() as usize;

            for (i, y) in (max_height-height..max_height-1).enumerate() {

                let starting_offset = offset + (max_width * y * char_count as i32) as usize + padding;

                pixels.splice(starting_offset..starting_offset + width as usize,
                     bmp[(i as i32 * width) as usize..(i as i32 *width + width) as usize].iter().cloned());
            }

            offset_table.extend_from_slice(&[xoff as f32 / width as f32, yoff as f32 / height as f32]);
            offset_table.extend_from_slice(&[0f32; 2]);
        }

        let mut command_buffer = CommandBuffer::new(device, command_pool, false);
        command_buffer.begin(device, &vk::CommandBufferInheritanceInfo::default(), vk::CommandBufferUsageFlags::empty());

        let tex = Texture::new_raw_data(
            device,
            &mut command_buffer,
            pixels.as_slice(),
            max_width as u32 * char_count,
            max_height as u32,
            vk::Format::R8_SRGB).expect("Failed to create the font atlas texture");

        let offset_table_buffer = Buffer::new(
            device,
            &mut command_buffer,
            unsafe {from_raw_parts(offset_table.as_ptr() as *const u8, offset_table.len() * 4)},
            BufferType::Uniform,
            false);

        command_buffer.end(device);



        let fence = Fence::new(device, false);

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer.get_command_buffer(),
            ..Default::default()
        };

        unsafe{
            device.get_ash_device().queue_submit(device.get_queue(), &[submit_info], fence.get_fence())
            .expect("Failed to submit the font atlas command buffer");

            device.get_ash_device().wait_for_fences(&[fence.get_fence()], true, std::u64::MAX)
            .expect("Failed to waitd for the font atlas load fence");
        }

        command_buffer.cleanup(device);

        (tex, offset_table_buffer)
    }

    pub fn get_data_for_str(&self, string: &str) -> Vec<u8> {
        let mut dat = Vec::<u8>::with_capacity(string.len());

        for c in string.chars() {
            //let char_dat = self.char_data[c as usize - 32];

            if c as u32 == 32 {// space
                dat.push(255);
                continue;
            } 

            dat.push(c as u8 - self.starting_offset as u8);
        }

        dat
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

        let push_constants = [self.char_count.to_ne_bytes(), info.scale.to_ne_bytes(), info.pos.0.to_ne_bytes(), info.pos.1.to_ne_bytes()].concat();

        unsafe{

            device.get_ash_device().cmd_push_constants(command_buffer.get_command_buffer(), render_pass.get_layout(), vk::ShaderStageFlags::ALL,
             4, &push_constants);

            device.get_ash_device().cmd_bind_vertex_buffers(command_buffer.get_command_buffer(), 1,
             &[data_buffer.get_buffer()], &[0]);

            device.get_ash_device().cmd_draw_indexed(command_buffer.get_command_buffer(), 6, info.char_count,
             0, 0, 0);
        }
    }

      pub fn get_descriptor_write_sets(
        &'a self,
        set: &'a DescriptorSet,
    ) -> ([vk::WriteDescriptorSet; 2], Pin<Box<[DescriptorInfo; 2]>>) {

        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.font_atlas_tex.get_image_view(),
            sampler: self.font_atlas_tex.get_sampler(),
        };

        let buffer_info = vk::DescriptorBufferInfo{
            buffer: self.offset_table_buffer.get_buffer(),
            offset: 0,
            range: vk::WHOLE_SIZE
        };


        let infos = Pin::new(Box::new(
            [DescriptorInfo::Image(vec![image_info]),
             DescriptorInfo::Buffer(vec![buffer_info])]));

        let image_descriptor_write = set.create_write_set(
            &infos.as_ref()[0],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            2, 
            1,
            1
        );

        let buffer_descriptor_write = set.create_write_set(
            &infos.as_ref()[1],
            vk::DescriptorType::UNIFORM_BUFFER,
            0, 
            1,
            7
        );

        ([image_descriptor_write, buffer_descriptor_write], infos)
    }

    pub fn destroy(&mut self, device: &Device) {
        self.font_atlas_tex.destroy(device);
        self.offset_table_buffer.destroy(device);
    }
}