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
    StaticDraw,
    // TODO rest of the usage enums
}

impl BufferUsage {
    pub fn gl_type(self) -> u32 {
        match self {
            BufferUsage::StaticDraw => gl::STATIC_DRAW,
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
}
