use std::rc::Rc;

use glow::HasContext;

use super::{Context, ElementBuffer, Error, Vertex, VertexBuffer, VertexDecls};

pub struct VertexArray<V, E = u32>
where
    V: VertexDecls,
{
    element_buffer: Rc<ElementBuffer<E>>,
    vertex_buffers: V::RcVertexBufferTuple,
    id: glow::VertexArray,
}

impl<V, E> VertexArray<V, E>
where
    V: Vertex,
{
    pub fn new(
        element_buffer: Rc<ElementBuffer<E>>,
        vertex_buffer: Rc<VertexBuffer<V>>,
    ) -> Result<Self, Error> {
        let gl = element_buffer.gl();
        let id = unsafe { gl.create_vertex_array() }.map_err(Error::Glow)?;

        unsafe {
            gl.bind_vertex_array(Some(id));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer.id()));
            V::bind_to_vertex_array(&*vertex_buffer, 0, 0);
            gl.bind_vertex_array(None);
        }

        Ok(Self {
            element_buffer,
            vertex_buffers: vertex_buffer,
            id,
        })
    }
}

impl<V, E> VertexArray<V, E>
where
    V: VertexDecls,
{
    pub fn new_instanced(
        element_buffer: Rc<ElementBuffer<E>>,
        vertex_buffers: V::RcVertexBufferTuple,
        divisors: &[u32],
    ) -> Result<Self, Error> {
        assert!(divisors.len() == V::N);

        let gl = element_buffer.gl();
        let id = unsafe { gl.create_vertex_array() }.map_err(Error::Glow)?;

        unsafe {
            gl.bind_vertex_array(Some(id));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer.id()));
            V::bind_to_vertex_array(vertex_buffers.clone(), divisors, 0);
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
