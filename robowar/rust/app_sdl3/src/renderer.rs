use std::rc::Rc;

use color_eyre::eyre::eyre;
use lib::geom::Vec2u32;

struct Texture<'a> {
    texture: sdl3::render::Texture<'a>,
}

impl<'a> lib::graphics::renderer::Texture for Texture<'a> {
    fn size(&self) -> lib::geom::Vec2u32 {
        Vec2u32::new(self.width(), self.height())
    }

    fn width(&self) -> u32 {
        self.texture.width()
    }

    fn height(&self) -> u32 {
        self.texture.height()
    }

    fn update(
        &mut self,
        size: lib::geom::Vec2u32,
        pixels: &[lib::graphics::color::RGBAu8],
        destination: lib::geom::Vec2u32,
    ) -> color_eyre::eyre::Result<()> {
        self.texture.update(
            Some(sdl3::rect::Rect::new(
                destination.x as i32,
                destination.y as i32,
                size.x,
                size.y,
            )),
            bytemuck::cast_slice(pixels),
            size.x as usize * 4,
        )?;
        Ok(())
    }
}

struct Renderer {
    canvas: sdl3::render::Canvas<sdl3::video::Window>,
    texture_creator: sdl3::render::TextureCreator<sdl3::video::WindowContext>,
}

impl Renderer {
    pub fn new(canvas: sdl3::render::Canvas<sdl3::video::Window>) -> Self {
        let texture_creator = canvas.texture_creator();
        Self {
            canvas,
            texture_creator,
        }
    }
}

impl<'texture> lib::graphics::renderer::Renderer for Renderer {
    type Texture
        = self::Texture<'texture>
    where
        Self: 'texture;

    fn create_texture_size(&mut self, size: Vec2u32) -> color_eyre::eyre::Result<Self::Texture> {
        let texture = self.texture_creator.create_texture(
            rgba(),
            sdl3::render::TextureAccess::Static,
            size.x,
            size.y,
        )?;
        Ok(Self::Texture { texture })
    }

    fn create_texture_pixels(
        &mut self,
        size: Vec2u32,
        pixels: &mut [lib::graphics::color::RGBAu8],
    ) -> color_eyre::eyre::Result<Self::Texture> {
        let surface = sdl3::surface::Surface::from_data(
            bytemuck::cast_slice_mut(pixels),
            size.x,
            size.y,
            size.x * 4,
            rgba(),
        )?;
        let texture = self.texture_creator.create_texture_from_surface(surface)?;
        Ok(Self::Texture { texture })
    }

    fn clear_screen(
        &mut self,
        clear_color: lib::graphics::color::RGBAf32,
    ) -> color_eyre::eyre::Result<()> {
        todo!()
    }

    fn set_ortho(&mut self, r: lib::geom::Rectf32) -> color_eyre::eyre::Result<()> {
        todo!()
    }

    fn fill_rect(
        &mut self,
        rect: lib::geom::Rectf32,
        transform: lib::geom::Affine2f32,
        material: &lib::graphics::renderer::Material<'_, Self::Texture>,
    ) -> color_eyre::eyre::Result<()> {
        todo!()
    }

    fn present() -> color_eyre::eyre::Result<()> {
        todo!()
    }
}

fn rgba() -> sdl3::pixels::PixelFormat {
    sdl3::pixels::PixelFormat::from_masks(sdl3::pixels::PixelMasks {
        bpp: 32,
        rmask: 0x000000ff,
        gmask: 0x0000ff00,
        bmask: 0x00ff0000,
        amask: 0xff000000,
    })
}
