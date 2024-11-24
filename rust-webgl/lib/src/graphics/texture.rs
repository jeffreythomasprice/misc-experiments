use super::colors::U8RGBA;
use crate::error::Error;
use image::{EncodableLayout, ImageFormat};
use log::*;
use nalgebra_glm::U32Vec2;
use std::{
    fmt::Debug,
    io::{BufRead, Seek},
    path::Path,
    rc::Rc,
};
use web_sys::{WebGl2RenderingContext, WebGlTexture};

pub struct Texture {
    context: Rc<WebGl2RenderingContext>,
    texture: WebGlTexture,
}

impl Texture {
    pub fn new_with_pixels(
        context: Rc<WebGl2RenderingContext>,
        size: U32Vec2,
        data: &[U8RGBA],
    ) -> Result<Self, Error> {
        let result = context
            .create_texture()
            .ok_or(format!("failed to create texture"))?;
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result));

        context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                size.x as i32,
                size.y as i32,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                bytemuck::cast_slice(data),
                0,
            )?;

        // TODO only if if power of 2 do mipmaps
        context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        // TODO if not a power of 2 texture to clamp, although this shouldn't be required in webgl2
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::REPEAT as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::REPEAT as i32,
        );

        // TODO should be nearest if not power of 2
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(Self {
            context,
            texture: result,
        })
    }

    pub fn new_with_image_data<P, R>(
        context: Rc<WebGl2RenderingContext>,
        path: Option<P>,
        r: R,
    ) -> Result<Self, Error>
    where
        P: Debug + AsRef<Path>,
        R: BufRead + Seek,
    {
        let mut image_reader = image::ImageReader::new(r);
        image_reader = match path {
            Some(path) => {
                image_reader = match ImageFormat::from_path(&path) {
                    Ok(format) => {
                        image_reader.set_format(format);
                        image_reader
                    }
                    Err(from_path_error) => {
                        image_reader.with_guessed_format().map_err(|from_guessed_format_error| {
                            format!("unable to determine image format, error checking path = {:?}, error guessing from bytes = {:?}", from_path_error,from_guessed_format_error)
                        })?
                    }
                }
            }
            None => image_reader.with_guessed_format().map_err(|e| {
                format!(
                    "unable to determine image format, error guessing from bytes = {:?}",
                    e
                )
            })?,
        };
        let image = image_reader
            .decode()
            .map_err(|e| format!("error decoding image, path={:?}, error={:?}", path, e))?
            .into_rgba8();
        Self::new_with_pixels(
            context,
            U32Vec2::new(image.width(), image.height()),
            bytemuck::cast_slice(image.as_bytes()),
        )
    }

    // TODO init from various html image types

    // TODO init from url

    pub fn bind(&self) {
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn bind_none(&self) {
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.context.delete_texture(Some(&self.texture))
    }
}
