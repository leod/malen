use std::{marker::PhantomData, rc::Rc};

use glow::HasContext;

use super::{Context, Error, Vertex};

pub struct VertexBuffer<V> {
    gl: Rc<Context>,
    pub(super) buffer: <glow::Context as HasContext>::Buffer,
    _phantom: PhantomData<V>,
}

impl<V> VertexBuffer<V>
where
    V: Vertex,
{
    pub fn new_dynamic(gl: Rc<Context>) -> Result<Self, Error> {
        let buffer = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        Ok(Self {
            gl,
            buffer,
            _phantom: PhantomData,
        })
    }

    pub fn new_static(gl: Rc<Context>, data: &[V]) -> Result<Self, Error> {
        let mut vertex_buffer = Self::new_dynamic(gl)?;
        vertex_buffer.set_data_with_usage(data, glow::STATIC_DRAW);

        Ok(vertex_buffer)
    }

    pub fn set_data(&self, data: &[V]) {
        // TODO: Prevent implicit synchronization somehow.
        // https://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf

        self.set_data_with_usage(data, glow::STREAM_DRAW);
    }

    fn set_data_with_usage(&self, data: &[V], usage: u32) {
        let data_u8 = bytemuck::cast_slice(data);
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer));
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, data_u8, usage);
        }
    }
}

impl<V> VertexBuffer<V> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }
}

impl<V> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
        }
    }
}
