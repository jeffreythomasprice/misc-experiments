use super::colors::U8RGBA;
use crate::{
    error::Error,
    math::{rect::Rect, size::Size, vec2::Vec2},
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
    pub fn max_size(context: &WebGl2RenderingContext) -> Result<usize, Error> {
        Ok(context
            .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)?
            .as_f64()
            .ok_or("expected max texture size to be a number".to_string())? as usize)
    }

    pub fn new_with_size(context: Rc<WebGl2RenderingContext>, size: Size<u32>) -> Result<Self, Error> {
        let result = Self::common_init(&context, size, |context| {
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

            // TODO try tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array

            Ok(())
        })?;

        Ok(Self {
            context,
            texture: result,
            size,
        })
    }

    pub fn new_with_pixels(context: Rc<WebGl2RenderingContext>, size: Size<u32>, data: &[U8RGBA]) -> Result<Self, Error> {
        let result = Self::common_init(&context, size, |context| {
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
    /// `destination` is the location on this texture to draw pixels to. Pixels that fall outside the bounds of this textuer are silently
    /// ignored.
    ///
    /// `source` is the bounding rectangle inside the source image to draw from. Pixels that lie outside the source image are silently
    /// ignored.
    ///
    /// `pixels` and `pixel_size` are the whole image to pull pixels from.
    ///
    pub fn copy_pixels(
        &mut self,
        destination: &Vec2<u32>,
        source: &Rect<u32>,
        pixels_size: &Size<u32>,
        pixels: &[U8RGBA],
    ) -> Result<(), Error> {
        // where we want to pull pixels from
        let source_bounds = source;
        let destination_bounds = Rect::with_position_and_size(*destination, source_bounds.size());

        // the actual limits of where we can pull pixels from and draw to
        let pixels_bounds = Rect::with_position_and_size(Vec2::zeroes(), *pixels_size);
        let texture_bounds = Rect::with_position_and_size(Vec2::zeroes(), *self.size());

        // clip the bounds we care about to those limits
        let source_bounds = source_bounds.intersect(&pixels_bounds);
        let destination_bounds = destination_bounds.intersect(&texture_bounds);

        match (source_bounds, destination_bounds) {
            // we have at least one actual pixel to draw
            (Some(source_bounds), Some(destination_bounds)) => {
                // further clip to the smaller of the two sizes
                // this could happen if the previous intersection clipped one of them
                // we can only draw the smaller region
                let (source_bounds, destination_bounds) = if source_bounds.size() != destination_bounds.size() {
                    let smaller_size = Size {
                        width: source_bounds.size().width.min(destination_bounds.size().width),
                        height: source_bounds.size().height.min(destination_bounds.size().height),
                    };
                    (
                        Rect::with_position_and_size(*source_bounds.origin(), smaller_size),
                        Rect::with_position_and_size(*destination_bounds.origin(), smaller_size),
                    )
                } else {
                    (source_bounds, destination_bounds)
                };

                self.bind();

                // we can only copy the whole pixels array at once
                // so if we're trying to copy something other than the whole thing at once we have to go row by row
                if source_bounds != pixels_bounds {
                    for y in 0..source_bounds.size().height {
                        self.context
                            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                                WebGl2RenderingContext::TEXTURE_2D,
                                0,
                                destination_bounds.origin().x as i32,
                                (y + destination_bounds.origin().y) as i32,
                                source_bounds.size().width as i32,
                                1,
                                WebGl2RenderingContext::RGBA,
                                WebGl2RenderingContext::UNSIGNED_BYTE,
                                bytemuck::cast_slice(pixels),
                                0,
                            )
                            .map_err(|e| format!("error copying pixels to texture region, row={}: {:?}", y, e))?;
                    }
                } else {
                    self.context
                        .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                            WebGl2RenderingContext::TEXTURE_2D,
                            0,
                            destination_bounds.origin().x as i32,
                            destination_bounds.origin().y as i32,
                            pixels_size.width as i32,
                            pixels_size.height as i32,
                            WebGl2RenderingContext::RGBA,
                            WebGl2RenderingContext::UNSIGNED_BYTE,
                            bytemuck::cast_slice(pixels),
                            0,
                        )
                        .map_err(|e| format!("error copying pixels to texture region: {e:?}"))?;
                };

                // TODO mark as dirty instead of regenerating mipmaps
                self.context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

                self.bind_none();
            }
            // at least one end is out of bounds, so nothing to do
            _ => (),
        };
        Ok(())
    }

    fn common_init<F>(context: &WebGl2RenderingContext, size: Size<u32>, f: F) -> Result<web_sys::WebGlTexture, Error>
    where
        F: FnOnce(&WebGl2RenderingContext) -> Result<(), Error>,
    {
        let result = context.create_texture().ok_or("failed to create texture".to_string())?;
        context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&result));

        if let Err(e) = f(context) {
            context.delete_texture(Some(&result));
            Err(e)?;
        }

        if size.width >= 1 && size.height >= 1 {
            context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
        }

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
            WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR as i32,
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
