use std::{ffi::c_void, marker::PhantomData};

use bytemuck::Pod;
use color_eyre::eyre::Result;

#[derive(Debug, Clone, Copy)]
pub enum BufferTarget {
    Array,
    ElementArray,
}

impl BufferTarget {
    pub fn gl_type(self) -> u32 {
        match self {
            BufferTarget::Array => gl::ARRAY_BUFFER,
            BufferTarget::ElementArray => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BufferUsage {
    StreamDraw,
    StreamRead,
    StreamCopy,
    StaticDraw,
    StaticRead,
    StaticCopy,
    DynamicDraw,
    DynamicRead,
    DynamicCopy,
}

impl BufferUsage {
    pub fn gl_type(self) -> u32 {
        match self {
            Self::StreamDraw => gl::STREAM_DRAW,
            Self::StreamRead => gl::STREAM_READ,
            Self::StreamCopy => gl::STREAM_COPY,
            Self::StaticDraw => gl::STATIC_DRAW,
            Self::StaticRead => gl::STATIC_READ,
            Self::StaticCopy => gl::STATIC_COPY,
            Self::DynamicDraw => gl::DYNAMIC_DRAW,
            Self::DynamicRead => gl::DYNAMIC_READ,
            Self::DynamicCopy => gl::DYNAMIC_COPY,
        }
    }
}

pub struct Buffer<T> {
    target: BufferTarget,
    instance: u32,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(target: BufferTarget, usage: BufferUsage, data: &[T]) -> Result<Self>
    where
        T: Pod,
    {
        unsafe {
            let mut instance = 0;
            gl::GenBuffers(1, &mut instance);

            gl::BindBuffer(target.gl_type(), instance);

            let bytes: &[u8] = bytemuck::cast_slice(data);
            gl::BufferData(
                target.gl_type(),
                bytes.len() as isize,
                data.as_ptr() as *mut c_void,
                usage.gl_type(),
            );

            gl::BindBuffer(target.gl_type(), 0);

            Ok(Self {
                target,
                instance,
                len: data.len(),
                _phantom: Default::default(),
            })
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.target.gl_type(), self.instance);
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set_data(&mut self, offset: usize, data: &[T]) -> Result<()>
    where
        T: Pod,
    {
        // TODO respect offset
        self.bind();
        let bytes: &[u8] = bytemuck::cast_slice(data);
        unsafe {
            gl::BufferSubData(
                self.target.gl_type(),
                0,
                bytes.len() as isize,
                data.as_ptr() as *mut c_void,
            );
            gl::BindBuffer(self.target.gl_type(), 0);
        }
        Ok(())
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.instance);
        };
    }
}
