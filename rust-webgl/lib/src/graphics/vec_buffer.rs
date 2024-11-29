use std::rc::Rc;

use bytemuck::Pod;
use web_sys::WebGl2RenderingContext;

use crate::error::Error;

use super::{buffer::Buffer, buffer::BufferUsage};

/// Both endpoints inclusive
struct Range {
    pub min: usize,
    pub max: usize,
}

/// A Vec paired with a Buffer
///
/// Writes go to the Vec, i.e. in local memory. When needed, any modified regions are synced to the Buffer, i.e. into video memory.
pub struct VecBuffer<T>
where
    T: Pod,
{
    vec: Vec<T>,
    buffer: Buffer<T>,

    dirty_range: Option<Range>,
}

impl<T> VecBuffer<T>
where
    T: Pod,
{
    pub fn new_array_buffer(context: Rc<WebGl2RenderingContext>, usage: BufferUsage) -> Result<Self, Error> {
        Self::new_array_buffer_with_capacity(context, usage, 0)
    }

    pub fn new_array_buffer_with_capacity(context: Rc<WebGl2RenderingContext>, usage: BufferUsage, cap: usize) -> Result<Self, Error> {
        Ok(Self {
            vec: Vec::with_capacity(cap),
            buffer: Buffer::new_array_buffer_with_len(context, usage, cap)?,

            dirty_range: None,
        })
    }

    pub fn new_array_buffer_from_vec(context: Rc<WebGl2RenderingContext>, usage: BufferUsage, data: Vec<T>) -> Result<Self, Error> {
        let buffer = Buffer::new_array_buffer_with_data(context, usage, &data)?;
        Ok(Self {
            vec: data,
            buffer,

            dirty_range: None,
        })
    }
}

impl VecBuffer<u16> {
    pub fn new_element_array_buffer(context: Rc<WebGl2RenderingContext>, usage: BufferUsage) -> Result<Self, Error> {
        Self::new_element_array_buffer_with_capacity(context, usage, 0)
    }

    pub fn new_element_array_buffer_with_capacity(
        context: Rc<WebGl2RenderingContext>,
        usage: BufferUsage,
        cap: usize,
    ) -> Result<Self, Error> {
        Ok(Self {
            vec: Vec::with_capacity(cap),
            buffer: Buffer::new_element_array_buffer_with_len(context, usage, cap)?,

            dirty_range: None,
        })
    }

    pub fn new_element_array_buffer_from_vec(
        context: Rc<WebGl2RenderingContext>,
        usage: BufferUsage,
        data: Vec<u16>,
    ) -> Result<Self, Error> {
        let buffer = Buffer::new_element_array_buffer_with_data(context, usage, &data)?;
        Ok(Self {
            vec: data,
            buffer,

            dirty_range: None,
        })
    }
}

impl<T> VecBuffer<T>
where
    T: Pod,
{
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn resize(&mut self, len: usize, value: T) {
        if len != self.vec.len() {
            self.vec.resize(len, value);
        }

        if len > self.buffer.len() {
            self.buffer.set_len(len);
            self.mark_all_dirty();
        }
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
        self.fix_buffer_len();
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.vec.shrink_to(min_capacity);
        self.fix_buffer_len();
    }

    pub fn push(&mut self, value: T) {
        self.vec.push(value);
        self.fix_buffer_len();
        self.mark_index_dirty(self.vec.len() - 1);
    }

    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.vec.extend_from_slice(slice);
        self.fix_buffer_len();
        self.mark_range_dirty(Range {
            min: self.vec.len() - slice.len(),
            max: self.vec.len() - 1,
        });
    }

    // TODO insert

    pub fn flush(&mut self) -> Result<(), Error> {
        if let Some(r) = self.dirty_range.take() {
            self.buffer.set(&self.vec[r.min..=r.max], r.min)?;
        }
        Ok(())
    }

    pub fn bind(&mut self) -> Result<(), Error> {
        self.flush()?;
        self.buffer.bind();
        Ok(())
    }

    pub fn bind_none(&self) {
        self.buffer.bind_none();
    }

    pub fn buffer_mut(&mut self) -> Result<&mut Buffer<T>, Error> {
        self.flush()?;
        Ok(&mut self.buffer)
    }

    fn fix_buffer_len(&mut self) {
        if self.vec.capacity() != self.buffer.len() {
            self.buffer.set_len(self.vec.capacity());
            self.mark_all_dirty();
        }
    }

    fn mark_all_dirty(&mut self) {
        self.dirty_range = Some(Range {
            min: 0,
            max: self.len() - 1,
        });
    }

    fn mark_index_dirty(&mut self, i: usize) {
        self.mark_range_dirty(Range { min: i, max: i });
    }

    fn mark_range_dirty(&mut self, new_range: Range) {
        self.dirty_range = match &self.dirty_range {
            Some(existing_range) => Some(Range {
                min: new_range.min.min(existing_range.min),
                max: new_range.max.max(existing_range.max),
            }),
            None => Some(new_range),
        };
    }
}

/*
TODO impl traits
Deref
DerefMut
Index
IndexMut
IntoIterator
*/
