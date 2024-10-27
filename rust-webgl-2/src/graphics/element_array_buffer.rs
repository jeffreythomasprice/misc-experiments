use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::error::Error;

use super::buffer_usage::BufferUsage;

pub struct ElementArrayBuffer {
    context: Rc<WebGl2RenderingContext>,
    usage: BufferUsage,
    gl_usage: u32,
    len: usize,
    stride: usize,
    buffer: WebGlBuffer,
}

impl ElementArrayBuffer {
    pub fn new_with_len(
        context: Rc<WebGl2RenderingContext>,
        usage: BufferUsage,
        len: usize,
    ) -> Result<Self, Error> {
        let buffer = context.create_buffer().ok_or("failed to create buffer")?;

        let gl_usage = usage.gl_usage();

        let stride = size_of::<u16>();

        let result = Self {
            context,
            usage,
            gl_usage,
            len,
            stride,
            buffer,
        };

        result.bind();
        result.context.buffer_data_with_i32(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            (result.len * result.stride) as i32,
            result.gl_usage,
        );
        result.bind_none();

        Ok(result)
    }

    pub fn new_with_data(
        context: Rc<WebGl2RenderingContext>,
        usage: BufferUsage,
        source: &[u16],
    ) -> Result<Self, Error> {
        let mut result = Self::new_with_len(context, usage, source.len())?;
        result.set(source, 0)?;
        Ok(result)
    }

    pub fn bind(&self) {
        self.context.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.buffer),
        );
    }

    pub fn bind_none(&self) {
        self.context
            .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set(&mut self, source: &[u16], index: usize) -> Result<(), Error> {
        let one_past_last = index + source.len();
        if one_past_last > self.len {
            return Err(format!("trying to copy out of bounds, size = {}, source length = {}, trying to place at index = {}", self.len, source.len(), index))?;
        }

        self.bind();
        self.context.buffer_sub_data_with_i32_and_u8_array(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            (index * self.stride) as i32,
            bytemuck::cast_slice(source),
        );
        self.bind_none();

        Ok(())
    }
}

impl Drop for ElementArrayBuffer {
    fn drop(&mut self) {
        self.context.delete_buffer(Some(&self.buffer))
    }
}
