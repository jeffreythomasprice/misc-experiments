use std::ffi::c_void;

use color_eyre::eyre::{Result, eyre};
use glam::{USizeVec2, usizevec2};
use load_image::{Image, export::rgb::RGBA8};
use sdl3::{pixels::PixelFormat, surface::Surface};
use tracing::*;

use crate::rect::USizeRect;

pub struct RGBA8Pixels<'a> {
    data: &'a [u8],
    size: USizeVec2,
    stride: usize,
}

impl<'a> RGBA8Pixels<'a> {
    /// * data - must be exactly stride * height bytes long
    /// * width - number of pixels per row
    /// * height - number of pixels per column
    /// * stride - number of bytes per row
    pub fn new(data: &'a [u8], size: USizeVec2, stride: usize) -> Result<Self> {
        let expected_len = stride * size.y;
        if data.len() != expected_len {
            Err(eyre!(
                "input was the wrong size, expected stride ({}) * height ({})  = {} bytes, got {}",
                stride,
                size.y,
                expected_len,
                data.len()
            ))?;
        }
        Ok(Self { data, size, stride })
    }

    pub fn data(&self) -> &[u8] {
        self.data
    }

    pub fn size(&self) -> &USizeVec2 {
        &self.size
    }

    pub fn width(&self) -> usize {
        self.size.x
    }

    pub fn height(&self) -> usize {
        self.size.y
    }

    pub fn stride(&self) -> usize {
        self.stride
    }
}

pub struct Texture {
    size: USizeVec2,
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
        Self::new_rgba8(&RGBA8Pixels::new(
            bytemuck::cast_slice(rgba_bytes),
            usizevec2(width, height),
            stride * 4,
        )?)
    }

    pub fn new_surface(surface: Surface) -> Result<Self> {
        let surface = surface.convert_format(PixelFormat::RGBA8888)?;
        Ok(surface.with_lock(|bytes| {
            Self::new_rgba8(&RGBA8Pixels::new(
                bytes,
                usizevec2(surface.width() as usize, surface.height() as usize),
                surface.pitch() as usize,
            )?)
        })?)
    }

    pub fn new_rgba8(data: &RGBA8Pixels) -> Result<Self> {
        trace!(
            "texture from RGBA size: ({} x {}), stride: {}",
            data.width(),
            data.height(),
            data.stride()
        );

        unsafe {
            let mut instance = 0;
            gl::GenTextures(1, &mut instance);
            gl::BindTexture(gl::TEXTURE_2D, instance);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, (data.stride() / 4) as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                data.width() as i32,
                data.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.data().as_ptr() as *const c_void,
            );

            if data.width().is_power_of_two() && data.height().is_power_of_two() {
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
            gl::BindTexture(gl::TEXTURE_2D, 0);

            Ok(Self {
                size: *data.size(),
                instance,
            })
        }
    }

    pub fn new_size(size: USizeVec2) -> Result<Self> {
        trace!("texture from size: ({} x {})", size.x, size.y,);

        // TODO de-duplicate with the initializer that does take initial contents
        unsafe {
            let mut instance = 0;
            gl::GenTextures(1, &mut instance);
            gl::BindTexture(gl::TEXTURE_2D, instance);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                size.x as i32,
                size.y as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            if size.x.is_power_of_two() && size.y.is_power_of_two() {
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

            gl::BindTexture(gl::TEXTURE_2D, 0);

            Ok(Self { size, instance })
        }
    }

    pub fn size(&self) -> &USizeVec2 {
        &self.size
    }

    pub fn width(&self) -> usize {
        self.size.x
    }

    pub fn height(&self) -> usize {
        self.size.y
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.instance);
        }
    }

    pub fn blit_surface(&mut self, src: &Surface, dst: &USizeVec2) -> Result<()> {
        let src = src.convert_format(PixelFormat::RGBA8888)?;
        Ok(src.with_lock(|bytes| -> Result<()> {
            self.blit_rgba8(
                &RGBA8Pixels::new(
                    bytes,
                    usizevec2(src.width() as usize, src.height() as usize),
                    src.pitch() as usize,
                )?,
                dst,
            )?;
            Ok(())
        })?)
    }

    pub fn blit_rgba8(&mut self, src: &RGBA8Pixels, dst: &USizeVec2) -> Result<()> {
        unsafe {
            self.bind();
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, (src.stride() / 4) as i32);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                dst.x as i32,
                dst.y as i32,
                src.width() as i32,
                src.height() as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                src.data().as_ptr() as *const c_void,
            );

            // TODO generate mipmaps?

            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Ok(())
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.instance);
        }
    }
}
