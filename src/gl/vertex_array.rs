use std::rc::Rc;

use glow::HasContext;

use super::{Context, ElementBuffer, Error, VertexDecls};

pub struct VertexArray<V, E>
where
    V: VertexDecls,
{
    element_buffer: Rc<ElementBuffer<E>>,
    vertex_buffers: V::RcVertexBufferTuple,
    id: glow::VertexArray,
}

impl<V, E> VertexArray<V, E>
where
    V: VertexDecls,
{
    pub fn new(
        element_buffer: Rc<ElementBuffer<E>>,
        vertex_buffers: V::RcVertexBufferTuple,
    ) -> Result<Self, Error> {
        let gl = element_buffer.gl();
        let id = unsafe { gl.create_vertex_array() }.map_err(Error::Glow)?;

        unsafe {
            gl.bind_vertex_array(Some(id));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer.id()));
            V::bind_to_vertex_array(&*gl, vertex_buffers.clone(), 0);
            gl.bind_vertex_array(None);
        }

        Ok(Self {
            element_buffer,
            vertex_buffers,
            id,
        })
    }
}

impl<V, E> VertexArray<V, E>
where
    V: VertexDecls,
{
    pub fn gl(&self) -> Rc<Context> {
        self.element_buffer.gl()
    }

    pub fn element_buffer(&self) -> Rc<ElementBuffer<E>> {
        self.element_buffer.clone()
    }

    pub fn vertex_buffers(&self) -> V::RcVertexBufferTuple {
        self.vertex_buffers.clone()
    }

    pub fn id(&self) -> glow::VertexArray {
        self.id
    }

    pub fn bind(&self) {
        unsafe {
            self.gl().bind_vertex_array(Some(self.id));
        }
    }
}

impl<V, E> Drop for VertexArray<V, E>
where
    V: VertexDecls,
{
    fn drop(&mut self) {
        unsafe {
            self.gl().delete_vertex_array(self.id);
        }
    }
}
