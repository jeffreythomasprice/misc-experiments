use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::errors::Result;

pub struct Texture {
    context: Rc<WebGl2RenderingContext>,
    target: u32,
    texture: WebGlTexture,
}

impl Texture {
    pub fn new_2d_rgba_u8(
        context: Rc<WebGl2RenderingContext>,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<Self> {
        let expected = (width * height * 4) as usize;
        if data.len() != expected {
            Err(format!("requested size is {width} x {height}, so expected pixel array is {expected} bytes, but got {}", data.len()))?;
        }

        let target = WebGl2RenderingContext::TEXTURE_2D;
        let texture = context.create_texture().ok_or("failed to create texture")?;

        context.bind_texture(target, Some(&texture));
        let result = context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                target,
                0,
                WebGl2RenderingContext::RGBA as i32,
                width as i32,
                height as i32,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                data,
                0,
            );
        if let Err(e) = result {
            context.delete_texture(Some(&texture));
            context.bind_texture(target, None);
            Err(e)?;
        }

        fn is_power_of_2(x: u32) -> bool {
            x & (x - 1) == 0
        }
        if is_power_of_2(width) && is_power_of_2(height) {
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR as i32,
            );
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::REPEAT as i32,
            );
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::REPEAT as i32,
            );
            context.generate_mipmap(target);
        } else {
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            context.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
        }

        context.bind_texture(target, None);

        Ok(Self {
            context,
            target,
            texture,
        })
    }

    // TODO tex_image_3d_with_u8_array_and_src_offset

    pub fn bind(&self) {
        self.context.bind_texture(self.target, Some(&self.texture))
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.context.delete_texture(Some(&self.texture))
    }
}
