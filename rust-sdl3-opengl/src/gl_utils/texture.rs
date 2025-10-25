use std::ffi::c_void;

use color_eyre::eyre::{Result, eyre};
use load_image::Image;
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
        Self::new_rgba8(bytemuck::cast_slice(rgba_bytes), width, height, stride)
    }

    /// * rgba_bytes - must be exactly stride * height bytes long
    /// * width - number of pixels per row
    /// * height - number of pixels per column
    /// * stride - number of bytes per row
    pub fn new_rgba8(
        rgba_bytes: &[u8],
        width: usize,
        height: usize,
        stride: usize,
    ) -> Result<Self> {
        trace!(
            "texture from rgba size: ({} x {}), stride: {}",
            width, height, stride
        );
        let expected_len = stride * height * 4;
        if rgba_bytes.len() != expected_len {
            Err(eyre!(
                "input was the wrong size, expected stride ({}) * height ({}) * 4 = {} bytes, got {}",
                stride,
                height,
                expected_len,
                rgba_bytes.len()
            ))?;
        }

        unsafe {
            let mut instance = 0;
            gl::GenTextures(1, &mut instance);
            gl::BindTexture(gl::TEXTURE_2D, instance);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, stride as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                rgba_bytes.as_ptr() as *const c_void,
            );
            // TODO alternate wrap and filter modes, no mipmaps, depending on whether sizes are powers of 2
            gl::GenerateMipmap(gl::TEXTURE_2D);
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
