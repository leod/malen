use std::{cell::Cell, marker::PhantomData, rc::Rc};

use glow::HasContext;

use super::{Context, Error, Vertex};

pub struct VertexBuffer<V> {
    gl: Rc<Context>,
    id: glow::Buffer,
    len: Cell<usize>,
    _phantom: PhantomData<V>,
}

impl<V> VertexBuffer<V>
where
    V: Vertex,
{
    pub fn new(gl: Rc<Context>) -> Result<Self, Error> {
        let id = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        Ok(Self {
            gl,
            id,
            len: Cell::new(0),
            _phantom: PhantomData,
        })
    }

    pub fn new_static(gl: Rc<Context>, data: &[V]) -> Result<Self, Error> {
        let vertex_buffer = Self::new(gl)?;
        vertex_buffer.set_data_with_usage(data, glow::STATIC_DRAW);

        Ok(vertex_buffer)
    }

    pub fn set(&self, data: &[V]) {
        // TODO: Prevent implicit synchronization somehow.
        // https://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf

        self.set_data_with_usage(data, glow::STREAM_DRAW);
    }

    fn set_data_with_usage(&self, data: &[V], usage: u32) {
        let data_u8 = bytemuck::cast_slice(data);

        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, data_u8, usage);
        }

        self.len.set(data.len());
    }
}

impl<V> VertexBuffer<V> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn id(&self) -> glow::Buffer {
        self.id
    }

    pub fn len(&self) -> usize {
        self.len.get()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<V> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.id);
        }
    }
}
