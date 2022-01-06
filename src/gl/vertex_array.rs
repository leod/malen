use std::rc::Rc;

use glow::HasContext;

use super::{AttributeValueType, Context, ElementBuffer, Error, Vertex, VertexBuffer};

pub struct VertexArray<V, E> {
    vertex_buffer: Rc<VertexBuffer<V>>,
    element_buffer: Rc<ElementBuffer<E>>,
    id: glow::VertexArray,
}

impl<V, E> VertexArray<V, E>
where
    V: Vertex,
{
    pub fn new(
        vertex_buffer: Rc<VertexBuffer<V>>,
        element_buffer: Rc<ElementBuffer<E>>,
    ) -> Result<Self, Error> {
        let gl = vertex_buffer.gl();
        assert!(Rc::ptr_eq(&gl, &element_buffer.gl()));

        let id = unsafe { vertex_buffer.gl().create_vertex_array() }.map_err(Error::Glow)?;

        unsafe {
            gl.bind_vertex_array(Some(id));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer.id()));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer.id()));
            V::bind_to_vertex_array(&*gl, 0);
        }

        unsafe {
            gl.bind_vertex_array(None);
        }

        Ok(Self {
            vertex_buffer,
            element_buffer,
            id,
        })
    }
}

impl<V, E> VertexArray<V, E> {
    pub fn gl(&self) -> Rc<Context> {
        self.vertex_buffer.gl()
    }

    pub fn vertex_buffer(&self) -> Rc<VertexBuffer<V>> {
        self.vertex_buffer.clone()
    }

    pub fn element_buffer(&self) -> Rc<ElementBuffer<E>> {
        self.element_buffer.clone()
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

impl<V, E> Drop for VertexArray<V, E> {
    fn drop(&mut self) {
        unsafe {
            self.gl().delete_vertex_array(self.id);
        }
    }
}
