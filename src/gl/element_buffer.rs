use std::{marker::PhantomData, rc::Rc};

use bytemuck::Pod;
use glow::HasContext;

use super::{Context, Error};

pub trait Element: Pod {}

impl Element for u32 {}

impl Element for u16 {}

pub struct ElementBuffer<E> {
    gl: Rc<Context>,
    buffer: <glow::Context as HasContext>::Buffer,
    _phantom: PhantomData<E>,
}

impl<E: Element> ElementBuffer<E> {
    pub fn new_dynamic(gl: Rc<Context>) -> Result<Self, Error> {
        let buffer = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(buffer));
        }

        Ok(Self {
            gl,
            buffer,
            _phantom: PhantomData,
        })
    }

    pub fn new_static(gl: Rc<Context>, data: &[E]) -> Result<Self, Error> {
        let vertex_buffer = Self::new_dynamic(gl)?;

        let data_u8 = bytemuck::cast_slice(data);
        unsafe {
            vertex_buffer.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                data_u8,
                glow::STATIC_DRAW,
            );
        }

        Ok(vertex_buffer)
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
    }
}

impl<V> Drop for ElementBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
        }
    }
}
