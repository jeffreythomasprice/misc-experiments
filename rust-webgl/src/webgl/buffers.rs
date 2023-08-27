use std::rc::Rc;

use js_sys::{Float32Array, Uint8Array};
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::errors::Result;

pub struct Buffer {
    context: Rc<WebGl2RenderingContext>,
    buffer: WebGlBuffer,
}

impl Buffer {
    pub fn new_with_f32(
        context: Rc<WebGl2RenderingContext>,
        target: u32,
        src_data: &[f32],
        usage: u32,
    ) -> Result<Self> {
        Self::new_with_callback(context.clone(), move || {
            // js typed array views are unsafe
            // if we do any allocations whlie that view is held we might move that data around in memory, invalidating that view
            unsafe {
                let view = Float32Array::view(src_data);
                context.buffer_data_with_array_buffer_view(target, &view, usage);
            }
            Ok(())
        })
    }

    pub fn new_with_u8(
        context: Rc<WebGl2RenderingContext>,
        target: u32,
        src_data: &[u8],
        usage: u32,
    ) -> Result<Self> {
        Self::new_with_callback(context.clone(), move || {
            // js typed array views are unsafe
            // if we do any allocations whlie that view is held we might move that data around in memory, invalidating that view
            unsafe {
                let view = Uint8Array::view(src_data);
                context.buffer_data_with_array_buffer_view(target, &view, usage)
            }
            Ok(())
        })
    }

    pub fn new_with_typed<T>(
        context: Rc<WebGl2RenderingContext>,
        target: u32,
        src_data: &[T],
        usage: u32,
    ) -> Result<Self> {
        unsafe {
            Self::new_with_u8(
                context,
                target,
                core::slice::from_raw_parts(
                    src_data.as_ptr() as *const u8,
                    std::mem::size_of_val(src_data),
                ),
                usage,
            )
        }
    }

    pub fn bind(&self) {
        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));
    }

    fn new_with_callback<F>(context: Rc<WebGl2RenderingContext>, f: F) -> Result<Self>
    where
        F: FnOnce() -> Result<()>,
    {
        let buffer = context.create_buffer().ok_or("failed to create buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        if let Err(e) = f() {
            context.delete_buffer(Some(&buffer));
            Err(e)?;
        }
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        Ok(Self { context, buffer })
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.context.delete_buffer(Some(&self.buffer))
    }
}
