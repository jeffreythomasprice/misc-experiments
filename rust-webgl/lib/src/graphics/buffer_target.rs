use web_sys::WebGl2RenderingContext;

#[derive(Debug)]
pub enum BufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
}

impl BufferTarget {
    pub fn gl_usage(&self) -> u32 {
        match self {
            BufferTarget::ArrayBuffer => WebGl2RenderingContext::ARRAY_BUFFER,
            BufferTarget::ElementArrayBuffer => WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
        }
    }
}
