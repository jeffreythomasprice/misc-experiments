use super::colors::U8RGBA;
use crate::{
    error::Error,
    math::{rect::Rect, size::Size},
};
use image::{EncodableLayout, ImageFormat};
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
    size: Size<u32>,
}

impl Texture {
    pub fn new_with_size(context: Rc<WebGl2RenderingContext>, size: Size<u32>) -> Result<Self, Error> {
        let result = Self::common_init(&context, |context| {
            /*
            If you don't provide any initial data you get this warning tryin to invoke texSubImage
            Texture has not been initialized prior to a partial upload, forcing the browser to clear it. This may be slow.
            */
            context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                size.width as i32,
                size.height as i32,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                &vec![0u8; (size.width * size.height * 4) as usize],
                0,
            )?;

            Ok(())
        })?;

        Ok(Self {
            context,
            texture: result,
            size,
        })
    }

    pub fn new_with_pixels(context: Rc<WebGl2RenderingContext>, size: Size<u32>, data: &[U8RGBA]) -> Result<Self, Error> {
        let result = Self::common_init(&context, |context| {
            context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                size.width as i32,
                size.height as i32,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                bytemuck::cast_slice(data),
                0,
            )?;
            Ok(())
        })?;

        Ok(Self {
            context,
            texture: result,
            size,
        })
    }

    /// Try to load the image from a file. Optionally give a path to the file so the format can be guessed from the file extension.
    pub fn new_with_image_data<P, R>(context: Rc<WebGl2RenderingContext>, path: Option<P>, r: R) -> Result<Self, Error>
    where
        P: Debug + AsRef<Path>,
        R: BufRead + Seek,
    {
        let mut image_reader = image::ImageReader::new(r);
        image_reader = match &path {
            Some(path) => match ImageFormat::from_path(path) {
                Ok(format) => {
                    image_reader.set_format(format);
                    image_reader
                }
                Err(from_path_error) => image_reader.with_guessed_format().map_err(|from_guessed_format_error| {
                    format!(
                        "unable to determine image format, error checking path = {:?}, error guessing from bytes = {:?}",
                        from_path_error, from_guessed_format_error
                    )
                })?,
            },
            None => image_reader
                .with_guessed_format()
                .map_err(|e| format!("unable to determine image format, error guessing from bytes = {:?}", e))?,
        };
        let image = image_reader
            .decode()
            .map_err(|e| format!("error decoding image, path={:?}, error={:?}", path, e))?
            .into_rgba8();
        Self::new_with_pixels(
            context,
            Size {
                width: image.width(),
                height: image.height(),
            },
            bytemuck::cast_slice(image.as_bytes()),
        )
    }

    // TODO init from various html image types

    pub fn bind(&self) {
        self.context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn bind_none(&self) {
        self.context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }

    pub fn size(&self) -> &Size<u32> {
        &self.size
    }

    /// Draws the given pixel array to the texture.
    ///
    /// Pixels aren't scaled.
    pub fn copy_pixels(
        &mut self,
        destination: &Rect<u32>,
        source: &Rect<u32>,
        source_size: Size<u32>,
        pixels: &[U8RGBA],
    ) -> Result<(), Error> {
        /*
        TODO actually copy sub images

        clip destination to (0,0) x this.size
        clip source to (0,0) x source_size

        min_size = min(destination.size, source.size)
        destination.size = min_size
        source.size = min_size
        */

        self.bind();
        self.context
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                destination.min.x as i32,
                destination.min.y as i32,
                source_size.width as i32,
                source_size.height as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                bytemuck::cast_slice(pixels),
                0,
            )
            .map_err(|e| format!("error copying pixels to texture region: {e:?}"))?;

        // TODO mark as dirty and regenerate mipmaps

        self.bind_none();
        Ok(())
    }

    fn common_init<F>(context: &WebGl2RenderingContext, f: F) -> Result<web_sys::WebGlTexture, Error>
    where
        F: FnOnce(&WebGl2RenderingContext) -> Result<(), Error>,
    {
        let result = context.create_texture().ok_or("failed to create texture".to_string())?;
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result));

        if let Err(e) = f(context) {
            // TODO delete texture
            Err(e)?;
        }

        // TODO power of 2 should do mipmaps
        // context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

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

        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            // TODO power of 2 should do mipmaps
            // WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR as i32,
            WebGl2RenderingContext::NEAREST as i32,
        );
        context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );

        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(result)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.context.delete_texture(Some(&self.texture))
    }
}
