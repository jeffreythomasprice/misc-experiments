use std::rc::Rc;

use color_eyre::eyre::Result;
use sdl3::{
    iostream::IOStream,
    pixels::{Color, PixelFormat},
    rect::Rect,
    surface::Surface,
    ttf::{self, Sdl3TtfContext},
};
use tracing::*;

use crate::gl_utils::texture::Texture;

pub struct Font {
    _ttf_context: Rc<Sdl3TtfContext>,
    font: ttf::Font<'static>,
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

    // TODO make a way to update a texture that already exists instead of always allocating a new one
    pub fn render_text(&self, s: &str) -> Result<Texture> {
        let color = Color::WHITE;
        let surfaces = s
            .split("\n")
            .map(|line| self.font.render(line).solid(color))
            .collect::<Result<Vec<_>, _>>()?;
        let mut width = 0;
        let mut height = 0;
        for s in surfaces.iter() {
            width = width.max(s.width());
            height += s.height();
        }
        let mut result = Surface::new(width, height, PixelFormat::RGBA8888)?;
        let mut y = 0;
        for s in surfaces.iter() {
            // TODO optional horizontal centering params
            s.blit(
                None,
                &mut result,
                Rect::new(0, y as i32, s.width(), s.height()),
            )?;
            y += s.height();
        }
        Ok(Texture::new_surface(result)?)
    }
}
