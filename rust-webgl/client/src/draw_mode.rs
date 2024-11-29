use web_sys::WebGl2RenderingContext;

pub enum DrawMode {
    Triangles,
    // TODO triangle fan, etc.
}

impl DrawMode {
    pub fn gl_constant(&self) -> u32 {
        match self {
            DrawMode::Triangles => WebGl2RenderingContext::TRIANGLES,
        }
    }
}
