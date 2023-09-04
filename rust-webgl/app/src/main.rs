#![feature(offset_of)]

use std::{rc::Rc, time::Duration};

use lib::{
    dom::game::{event_listeners::EventListener, event_state::EventState, main::launch},
    errors::Result,
    glmath::{
        angles::Degrees, fpscamera::FPSCamera, matrix4::Matrix4, rgba::Rgba, vector2::Vector2,
        vector3::Vector3,
    },
    webgl::{
        buffers::Buffer,
        shaders::ShaderProgram,
        textures::Texture,
        vertexarrays::{VertexArray, VertexArrayAttribute},
    },
};
use log::*;

use web_sys::WebGl2RenderingContext;

#[repr(C)]
#[derive(Debug)]
struct Vertex {
    position: Vector3<f32>,
    texture_coordinate: Vector2<f32>,
    color: Rgba<f32>,
}

struct State {
    context: Rc<WebGl2RenderingContext>,

    program: ShaderProgram,
    _buffer: Buffer,
    vertex_array: VertexArray,
    texture: Texture,

    ortho_matrix: Matrix4<f32>,
    perspective_matrix: Matrix4<f32>,
    camera: FPSCamera<f32>,
}

impl State {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self> {
        let context = Rc::new(context);

        let program = ShaderProgram::new(
            context.clone(),
            include_str!("shaders/shader.vertex"),
            include_str!("shaders/shader.fragment"),
        )?;

        let position_attribute = program.get_attribute("in_position")?;
        let texture_coordinate_attribute = program.get_attribute("in_texture_coordinate")?;
        let color_attribute = program.get_attribute("in_color")?;

        let buffer = Buffer::new_with_typed(
            context.clone(),
            WebGl2RenderingContext::ARRAY_BUFFER,
            &cube(
                Vector3::new(0f32, 0f32, 0f32),
                Vector3::new(2f32, 2f32, 2f32),
                Rgba::new(1f32, 1f32, 1f32, 1f32),
            ),
            WebGl2RenderingContext::STATIC_DRAW,
        )?;

        let vertex_array = VertexArray::new(
            context.clone(),
            &[
                VertexArrayAttribute {
                    shader_attribute: position_attribute,
                    buffer: &buffer,
                    size: 3,
                    type_: WebGl2RenderingContext::FLOAT,
                    normalized: false,
                    stride: std::mem::size_of::<Vertex>(),
                    offset: std::mem::offset_of!(Vertex, position),
                },
                VertexArrayAttribute {
                    shader_attribute: texture_coordinate_attribute,
                    buffer: &buffer,
                    size: 2,
                    type_: WebGl2RenderingContext::FLOAT,
                    normalized: false,
                    stride: std::mem::size_of::<Vertex>(),
                    offset: std::mem::offset_of!(Vertex, texture_coordinate),
                },
                VertexArrayAttribute {
                    shader_attribute: color_attribute,
                    buffer: &buffer,
                    size: 4,
                    type_: WebGl2RenderingContext::FLOAT,
                    normalized: false,
                    stride: std::mem::size_of::<Vertex>(),
                    offset: std::mem::offset_of!(Vertex, color),
                },
            ],
        )?;

        let texture_width = 256u32;
        let texture_height = 256u32;
        let mut pixels: Vec<Rgba<u8>> =
            Vec::with_capacity((texture_width as usize) * (texture_height as usize));
        for y in 0..texture_height {
            let a = ((y as f64) / (texture_height as f64) * 255f64) as u8;
            for x in 0..texture_width {
                let b = ((x as f64) / (texture_width as f64) * 255f64) as u8;
                pixels.push(Rgba::new(a, b, 255 - a, 255));
            }
        }
        let texture = unsafe {
            Texture::new_2d_rgba_u8(
                context.clone(),
                texture_width,
                texture_height,
                core::slice::from_raw_parts(
                    pixels.as_ptr() as *const u8,
                    std::mem::size_of_val(pixels.as_slice()),
                ),
            )?
        };

        let mut camera = FPSCamera::new(
            Vector3::new(4f32, 3f32, 4f32),
            Vector3::new(0f32, 0f32, 1f32),
            Vector3::new(0f32, 1f32, 0f32),
        );
        camera.look_at(Vector3::new(0f32, 0f32, 0f32));

        Ok(Self {
            context,

            program,
            _buffer: buffer,
            vertex_array,
            texture,

            ortho_matrix: Matrix4::new_identity(),
            perspective_matrix: Matrix4::new_identity(),
            camera,
        })
    }
}

impl EventListener for State {
    fn animate(&mut self, delta: Duration, event_state: &EventState) -> Result<()> {
        let key_left = event_state.is_keyboard_key_code_pressed(65)
            || event_state.is_keyboard_key_code_pressed(37);
        let key_right = event_state.is_keyboard_key_code_pressed(68)
            || event_state.is_keyboard_key_code_pressed(39);
        let key_up = event_state.is_keyboard_key_code_pressed(87)
            || event_state.is_keyboard_key_code_pressed(38);
        let key_down = event_state.is_keyboard_key_code_pressed(83)
            || event_state.is_keyboard_key_code_pressed(40);
        let key_raise = event_state.is_keyboard_key_code_pressed(32);
        let key_lower = event_state.is_keyboard_key_code_pressed(16);

        let forward = if key_up && !key_down {
            1f32
        } else if !key_up && key_down {
            -1f32
        } else {
            0f32
        };
        let right = if key_left && !key_right {
            -1f32
        } else if !key_left && key_right {
            1f32
        } else {
            0f32
        };
        let up = if key_raise && !key_lower {
            1f32
        } else if !key_raise && key_lower {
            -1f32
        } else {
            0f32
        };

        const SPEED: f32 = 5.0f32;
        self.camera.move_by(
            forward * delta.as_secs_f32() * SPEED,
            right * delta.as_secs_f32() * SPEED,
            up * delta.as_secs_f32() * SPEED,
        );

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.context.clear_color(0.25, 0.5, 0.75, 1.0);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.context.clear_depth(1.0f32);
        self.context.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.context.depth_func(WebGl2RenderingContext::LEQUAL);

        self.program.use_program();

        self.program
            .get_uniform("uniform_matrix")?
            .set_matrixf(self.perspective_matrix.clone().append(self.camera.matrix()));

        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE1);
        self.texture.bind();
        self.program.get_uniform("uniform_texture")?.set1i(1);

        self.vertex_array.bind();
        // TODO should be pulling the number of verticies to draw from the number of vertices or indices
        self.context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 36);
        self.context.bind_vertex_array(None);

        self.context.use_program(None);

        self.context.disable(WebGl2RenderingContext::DEPTH_TEST);

        Ok(())
    }

    fn resize(&mut self, size: Vector2<u32>) -> Result<()> {
        self.context.viewport(0, 0, size.x as i32, size.y as i32);

        self.ortho_matrix =
            Matrix4::new_ortho(0f32, size.x as f32, size.y as f32, 0f32, -1f32, 1f32);
        self.perspective_matrix = Matrix4::new_perspective(
            Degrees(65f32).into(),
            size.x as f32,
            size.y as f32,
            0.1f32,
            1000f32,
        );

        Ok(())
    }

    fn mousemove(
        &mut self,
        _event_state: &EventState,
        _location: Vector2<i32>,
        delta: Vector2<i32>,
        is_pointer_locked: bool,
    ) -> Result<()> {
        if is_pointer_locked {
            let delta = Vector2::new(delta.x as f32, delta.y as f32) / 5f32;
            self.camera
                .turn(Degrees(delta.y).into(), Degrees(delta.x).into())
        }

        Ok(())
    }

    fn mousedown(
        &mut self,
        _event_state: &EventState,
        _button: i16,
        _location: Vector2<i32>,
    ) -> Result<()> {
        Ok(())
    }

    fn mouseup(
        &mut self,
        _event_state: &EventState,
        _button: i16,
        _location: Vector2<i32>,
    ) -> Result<()> {
        Ok(())
    }

    fn keydown(&mut self, _event_state: &EventState, _key: String, _key_code: u32) -> Result<()> {
        Ok(())
    }

    fn keyup(&mut self, _event_state: &EventState, _key: String, _key_code: u32) -> Result<()> {
        Ok(())
    }
}

fn cube(center: Vector3<f32>, size: Vector3<f32>, color: Rgba<f32>) -> [Vertex; 36] {
    let half_size = size / 2f32;
    let min = center - half_size;
    let max = center + half_size;
    [
        // min z face
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // max z face
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // min y face
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // max y face
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // min x face
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(min.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        // max x face
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, min.z),
            texture_coordinate: Vector2::new(1f32, 0f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, max.y, max.z),
            texture_coordinate: Vector2::new(1f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, max.z),
            texture_coordinate: Vector2::new(0f32, 1f32),
            color,
        },
        Vertex {
            position: Vector3::new(max.x, min.y, min.z),
            texture_coordinate: Vector2::new(0f32, 0f32),
            color,
        },
    ]
}

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    if let Err(e) = launch(State::new) {
        error!("fatal error: {e:?}");
    }
}
