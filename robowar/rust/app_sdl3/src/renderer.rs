use std::rc::Rc;

use color_eyre::eyre::eyre;
use lib::{
    geom::{Rectf32, Vec2f32, Vec2u32},
    graphics::color::RGBAu8,
};

pub struct Texture {
    texture: sdl3::render::Texture<'static>,
}

impl Texture {
    fn new<'a>(texture: sdl3::render::Texture<'a>) -> Self {
        /*
        sdl enforces a particular lifetime on the texdture but does all it's own management via unsafe stuff so just let me own it for as
        long as I want
        */
        Self {
            texture: unsafe {
                // just to cast the lifetime
                std::mem::transmute(texture)
            },
        }
    }
}

impl lib::graphics::renderer::Texture for Texture {
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

pub struct Renderer {
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

    pub fn canvas(&self) -> &sdl3::render::Canvas<sdl3::video::Window> {
        &self.canvas
    }
}

impl lib::graphics::renderer::Renderer for Renderer {
    type Texture = self::Texture;

    fn create_texture_size(&mut self, size: Vec2u32) -> color_eyre::eyre::Result<Self::Texture> {
        let texture = self.texture_creator.create_texture(
            rgba(),
            sdl3::render::TextureAccess::Static,
            size.x,
            size.y,
        )?;
        Ok(Self::Texture::new(texture))
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
        Ok(Self::Texture::new(texture))
    }

    fn clear_screen(
        &mut self,
        clear_color: lib::graphics::color::RGBAf32,
    ) -> color_eyre::eyre::Result<()> {
        let clear_color: lib::graphics::color::RGBAu8 = clear_color.into();
        self.canvas.set_draw_color(sdl3::pixels::Color::RGBA(
            clear_color.red,
            clear_color.green,
            clear_color.blue,
            clear_color.alpha,
        ));
        self.canvas.clear();
        Ok(())
    }

    fn set_ortho(&mut self, r: lib::geom::Rectf32) -> color_eyre::eyre::Result<()> {
        todo!()
    }

    fn fill_rect(
        &mut self,
        rect: lib::geom::Rectf32,
        transform: lib::geom::Affine2f32,
        material: &mut lib::graphics::renderer::Material<'_, Self::Texture>,
    ) -> color_eyre::eyre::Result<()> {
        if let Some(texture) = &mut material.texture {
            texture.texture.set_blend_mode(if material.blend {
                sdl3::render::BlendMode::Blend
            } else {
                sdl3::render::BlendMode::None
            });
            if let Some(color) = material.color {
                let color: RGBAu8 = color.into();
                texture
                    .texture
                    .set_color_mod(color.red, color.green, color.blue);
                texture.texture.set_alpha_mod(color.alpha);
            }
            let scaled_rect = Rectf32::with_corners(
                Vec2f32::new(
                    rect.min().x * transform.scale.x,
                    rect.min().y * transform.scale.y,
                ),
                Vec2f32::new(
                    rect.max().x * transform.scale.x,
                    rect.max().y * transform.scale.y,
                ),
            );
            self.canvas.copy_ex(
                &texture.texture,
                // TODO texture src rect
                None,
                sdl3::render::FRect::new(
                    scaled_rect.min().x,
                    scaled_rect.min().y,
                    scaled_rect.width(),
                    scaled_rect.height(),
                ),
                transform.rotation as f64,
                sdl3::render::FPoint::new(0.0, 0.0),
                false,
                false,
            )?;
            if material.color.is_some() {
                texture.texture.set_color_mod(255, 255, 255);
                texture.texture.set_alpha_mod(255);
            }
        } else {
            // color only
            todo!()
        }
        Ok(())
    }

    fn present(&mut self) -> color_eyre::eyre::Result<()> {
        self.canvas.present();
        Ok(())
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
