use bytemuck::Pod;
use std::{marker::PhantomData, rc::Rc};
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::error::Error;

use super::{buffer_target::BufferTarget, buffer_usage::BufferUsage};

pub struct Buffer<T> {
    context: Rc<WebGl2RenderingContext>,
    gl_target: u32,
    gl_usage: u32,
    len: usize,
    stride: usize,
    buffer: WebGlBuffer,
    phantom: PhantomData<T>,
}

impl<T> Buffer<T>
where
    T: Pod,
{
    pub fn new_array_buffer_with_len(context: Rc<WebGl2RenderingContext>, usage: BufferUsage, len: usize) -> Result<Self, Error> {
        let buffer = context.create_buffer().ok_or("failed to create buffer")?;
        let mut result = Self {
            context,
            gl_target: BufferTarget::ArrayBuffer.gl_constant(),
            gl_usage: usage.gl_constant(),
            len: 0,
            stride: size_of::<T>(),
            buffer,
            phantom: PhantomData,
        };
        result.set_len(len);
        Ok(result)
    }

    pub fn new_array_buffer_with_data(context: Rc<WebGl2RenderingContext>, usage: BufferUsage, source: &[T]) -> Result<Self, Error> {
        let mut result = Self::new_array_buffer_with_len(context, usage, source.len())?;
        result.set(source, 0)?;
        Ok(result)
    }

    pub fn bind(&self) {
        self.context.bind_buffer(self.gl_target, Some(&self.buffer));
    }

    pub fn bind_none(&self) {
        self.context.bind_buffer(self.gl_target, None);
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set_len(&mut self, len: usize) {
        if self.len != len {
            self.len = len;
            self.bind();
            self.context
                .buffer_data_with_i32(self.gl_target, (self.len * self.stride) as i32, self.gl_usage);
            self.bind_none();
        }
    }

    pub fn set(&mut self, source: &[T], index: usize) -> Result<(), Error> {
        let one_past_last = index + source.len();
        if one_past_last > self.len {
            return Err(format!(
                "trying to copy out of bounds, size = {}, source length = {}, trying to place at index = {}",
                self.len,
                source.len(),
                index
            ))?;
        }

        self.bind();
        self.context
            .buffer_sub_data_with_i32_and_u8_array(self.gl_target, (index * self.stride) as i32, bytemuck::cast_slice(source));
        self.bind_none();

        Ok(())
    }
}

impl Buffer<u16> {
    pub fn new_element_array_buffer_with_len(context: Rc<WebGl2RenderingContext>, usage: BufferUsage, len: usize) -> Result<Self, Error> {
        let buffer = context.create_buffer().ok_or("failed to create buffer")?;
        let mut result = Self {
            context,
            gl_target: BufferTarget::ElementArrayBuffer.gl_constant(),
            gl_usage: usage.gl_constant(),
            len: 0,
            stride: size_of::<u16>(),
            buffer,
            phantom: PhantomData,
        };
        result.set_len(len);
        Ok(result)
    }

    pub fn new_element_array_buffer_with_data(
        context: Rc<WebGl2RenderingContext>,
        usage: BufferUsage,
        source: &[u16],
    ) -> Result<Self, Error> {
        let mut result = Self::new_element_array_buffer_with_len(context, usage, source.len())?;
        result.set(source, 0)?;
        Ok(result)
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        self.context.delete_buffer(Some(&self.buffer))
    }
}
