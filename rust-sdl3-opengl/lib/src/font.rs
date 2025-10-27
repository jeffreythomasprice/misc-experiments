use std::rc::Rc;

use color_eyre::eyre::{Result, eyre};
use glam::{USizeVec2, UVec2, usizevec2};
use sdl3::{
    iostream::IOStream,
    pixels::{Color, PixelFormat},
    rect::Rect,
    surface::Surface,
    ttf::{self, Sdl3TtfContext},
};
use tracing::*;

use crate::{gl_utils::texture::Texture, rect::USizeRect};

pub struct Font {
    _ttf_context: Rc<Sdl3TtfContext>,
    font: ttf::Font<'static>,
}

struct LineLayout<'text> {
    s: &'text str,
    bounds: USizeRect,
}

pub struct FontStringLayout<'font, 'text> {
    font: &'font Font,
    lines: Vec<LineLayout<'text>>,
    bounds: USizeRect,
}

impl Font {
    pub fn new_from_vec(
        ttf_context: Rc<Sdl3TtfContext>,
        vec: Vec<u8>,
        point_size: f32,
    ) -> Result<Self> {
        let font = ttf_context.load_font_from_iostream(IOStream::from_vec(vec)?, point_size)?;
        debug!(
            "loading font, family name: {:?}, style name: {:?}",
            font.face_family_name(),
            font.face_style_name()
        );
        Ok(Self {
            _ttf_context: ttf_context,
            font,
        })
    }

    pub fn layout<'font, 'text>(
        &'font self,
        s: &'text str,
        // TODO horizontal alignment parameter
    ) -> Result<FontStringLayout<'font, 'text>> {
        let mut width = 0;
        let mut height = 0;
        let mut lines = Vec::new();
        let mut y = 0;
        for s in s.split("\n") {
            let (render_width, render_height) = self.font.size_of(s)?;
            let render_width = render_width as usize;
            let render_height = render_height as usize;
            let line_spacing = self.font.recommended_line_spacing() as usize;
            width = width.max(render_width);
            height += line_spacing;
            lines.push(LineLayout {
                s,
                bounds: USizeRect::new_origin_size(
                    usizevec2(0, y),
                    usizevec2(render_width, render_height),
                ),
            });
            y += line_spacing;
        }
        Ok(FontStringLayout {
            font: self,
            lines,
            bounds: USizeRect::new_origin_size(usizevec2(0, 0), usizevec2(width, height)),
        })
    }
}

impl<'font, 'text> FontStringLayout<'font, 'text> {
    pub fn render_to_new_surface(&self, color: Color) -> Result<Surface> {
        let mut result = Surface::new(
            self.bounds.width() as u32,
            self.bounds.height() as u32,
            PixelFormat::RGBA8888,
        )?;
        for x in self.lines.iter() {
            self.font.font.render(x.s).solid(color)?.blit(
                None,
                &mut result,
                Rect::new(
                    x.bounds.origin().x as i32,
                    x.bounds.origin().y as i32,
                    x.bounds.width() as u32,
                    x.bounds.height() as u32,
                ),
            )?;
        }
        Ok(result)
    }

    /// If the given texture is None, or if it's too small to fit the rendered surface at the given location, reallocates the texture with
    /// a bigger size.
    ///
    /// Returns the updated or new texture.
    pub fn render_to_texture_resize_as_needed(
        &self,
        color: Color,
        texture: Option<Texture>,
        dst: USizeVec2,
    ) -> Result<Texture> {
        let surface = self.render_to_new_surface(color)?;

        let min_width = dst.x + surface.width() as usize;
        let min_height = dst.y + surface.height() as usize;

        let mut texture = if let Some(texture) = texture {
            if texture.width() < min_width || texture.height() < min_height {
                // texture is too small, make a new one
                info!(
                    "reallocating texture to new size: ({} x {})",
                    min_width, min_height
                );
                Texture::new_size(usizevec2(min_width, min_height))?
            } else {
                // texture is big enough
                texture
            }
        } else {
            // no texture exists
            info!(
                "allocating new dynamically sized texture: ({} x {})",
                min_width, min_height
            );
            Texture::new_size(usizevec2(min_width, min_height))?
        };

        texture.blit_surface(&surface, &dst)?;

        Ok(texture)
    }
}
