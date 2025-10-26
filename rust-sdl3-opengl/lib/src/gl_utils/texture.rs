use std::ffi::c_void;

use color_eyre::eyre::{Result, eyre};
use load_image::Image;
use sdl3::{pixels::PixelFormat, surface::Surface};
use tracing::*;

pub struct Texture {
    width: usize,
    height: usize,
    stride: usize,
    instance: u32,
}

impl Texture {
    pub fn new_image(image: Image) -> Result<Self> {
        trace!(
            "texture from image, size: ({} x {}), metadata: {:?}",
            image.width, image.height, image.meta
        );
        let (mut rgba, _) = image.into_rgba();
        let width = rgba.width();
        let (rgba_bytes, stride, height) = rgba.as_contiguous_buf();
        Self::new_rgba8(bytemuck::cast_slice(rgba_bytes), width, height, stride * 4)
    }

    pub fn new_surface(surface: Surface) -> Result<Self> {
        let surface = surface.convert_format(PixelFormat::RGBA8888)?;
        Ok(surface.with_lock(|bytes| {
            Self::new_rgba8(
                bytes,
                surface.width() as usize,
                surface.height() as usize,
                surface.pitch() as usize,
            )
        })?)
    }

    /// * bytes - must be exactly stride * height bytes long
    /// * width - number of pixels per row
    /// * height - number of pixels per column
    /// * stride - number of bytes per row
    pub fn new_rgba8(bytes: &[u8], width: usize, height: usize, stride: usize) -> Result<Self> {
        trace!(
            "texture from RGBA size: ({} x {}), stride: {}",
            width, height, stride
        );
        let expected_len = stride * height;
        if bytes.len() != expected_len {
            Err(eyre!(
                "input was the wrong size, expected stride ({}) * height ({})  = {} bytes, got {}",
                stride,
                height,
                expected_len,
                bytes.len()
            ))?;
        }

        unsafe {
            let mut instance = 0;
            gl::GenTextures(1, &mut instance);
            gl::BindTexture(gl::TEXTURE_2D, instance);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, (stride / 4) as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                bytes.as_ptr() as *const c_void,
            );

            if width.is_power_of_two() && height.is_power_of_two() {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::NEAREST_MIPMAP_LINEAR as i32,
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                gl::GenerateMipmap(gl::TEXTURE_2D);
            } else {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            }

            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
            Ok(Self {
                width,
                height,
                stride,
                instance,
            })
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.instance);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.instance);
        }
    }
}
