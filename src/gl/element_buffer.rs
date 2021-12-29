use std::{marker::PhantomData, rc::Rc};

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
    buffer: <glow::Context as HasContext>::Buffer,
    len: usize,
    _phantom: PhantomData<E>,
}

impl<E: Element> ElementBuffer<E> {
    pub fn new_dynamic(gl: Rc<Context>) -> Result<Self, Error> {
        let buffer = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        Ok(Self {
            gl,
            buffer,
            len: 0,
            _phantom: PhantomData,
        })
    }

    pub fn new_static(gl: Rc<Context>, data: &[E]) -> Result<Self, Error> {
        let buffer = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        let data_u8 = bytemuck::cast_slice(data);
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(buffer));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data_u8, glow::STATIC_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }

        Ok(Self {
            gl,
            buffer,
            len: data.len(),
            _phantom: PhantomData,
        })
    }

    pub fn set_data(&mut self, data: &[E]) {
        let data_u8 = bytemuck::cast_slice(data);

        // TODO: Prevent implicit synchronization somehow.
        // https://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf
        unsafe {
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.buffer));
            self.gl
                .buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data_u8, glow::STREAM_DRAW);
        }

        self.len = data.len();
    }
}

impl<E> ElementBuffer<E> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn bind(&self) {
        unsafe {
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.buffer));
        }
    }
}

impl<E> Drop for ElementBuffer<E> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
        }
    }
}
