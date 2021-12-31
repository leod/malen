use std::{cell::Cell, marker::PhantomData, rc::Rc};

use bytemuck::Pod;
use glow::HasContext;

use super::{Context, Error};

pub trait Element: Pod {
    fn to_gl() -> u32;
}

impl Element for u32 {
    fn to_gl() -> u32 {
        glow::UNSIGNED_INT
    }
}

impl Element for u16 {
    fn to_gl() -> u32 {
        glow::UNSIGNED_SHORT
    }
}

pub struct ElementBuffer<E> {
    gl: Rc<Context>,
    pub(super) buffer: <glow::Context as HasContext>::Buffer,
    len: Cell<usize>,
    _phantom: PhantomData<E>,
}

impl<E> ElementBuffer<E>
where
    E: Element,
{
    pub fn new_dynamic(gl: Rc<Context>) -> Result<Self, Error> {
        let buffer = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        Ok(Self {
            gl,
            buffer,
            len: Cell::new(0),
            _phantom: PhantomData,
        })
    }

    pub fn new_static(gl: Rc<Context>, data: &[E]) -> Result<Self, Error> {
        let mut vertex_buffer = Self::new_dynamic(gl)?;
        vertex_buffer.set_data_with_usage(data, glow::STATIC_DRAW);

        Ok(vertex_buffer)
    }

    pub fn set_data(&self, data: &[E]) {
        // TODO: Prevent implicit synchronization somehow.
        // https://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf

        self.set_data_with_usage(data, glow::STREAM_DRAW);
    }

    fn set_data_with_usage(&self, data: &[E], usage: u32) {
        let data_u8 = bytemuck::cast_slice(data);

        unsafe {
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.buffer));
            self.gl
                .buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data_u8, usage);
        }
    }
}

impl<E> ElementBuffer<E> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn len(&self) -> usize {
        self.len.get()
    }
}

impl<E> Drop for ElementBuffer<E> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
        }
    }
}