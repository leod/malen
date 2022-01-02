use std::rc::Rc;

use glow::HasContext;

use super::{AttributeValueType, Context, ElementBuffer, Error, Vertex, VertexBuffer};

pub struct VertexArray<V, E> {
    vertex_buffer: Rc<VertexBuffer<V>>,
    element_buffer: Rc<ElementBuffer<E>>,
    pub(super) vao: <glow::Context as HasContext>::VertexArray,
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

        let vao = unsafe { vertex_buffer.gl().create_vertex_array() }.map_err(Error::Glow)?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer.buffer));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer.buffer));
        }

        set_vertex_attribs::<V>(&*gl);

        unsafe {
            gl.bind_vertex_array(None);
        }

        Ok(Self {
            vertex_buffer,
            element_buffer,
            vao,
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

    pub fn bind(&self) {
        unsafe {
            self.gl().bind_vertex_array(Some(self.vao));
        }
    }
}

impl<V, E> Drop for VertexArray<V, E> {
    fn drop(&mut self) {
        unsafe {
            self.gl().delete_vertex_array(self.vao);
        }
    }
}

fn set_vertex_attribs<V: Vertex>(gl: &Context) {
    for (index, attribute) in V::attributes().iter().enumerate() {
        assert!(
            attribute.offset + attribute.num_elements * attribute.element_type.size()
                <= std::mem::size_of::<V>()
        );

        unsafe {
            gl.enable_vertex_attrib_array(index as u32);
        }

        match attribute.element_type {
            AttributeValueType::Float => unsafe {
                gl.vertex_attrib_pointer_f32(
                    index as u32,
                    attribute.num_elements as i32,
                    attribute.element_type.to_gl(),
                    false,
                    std::mem::size_of::<V>() as i32,
                    attribute.offset as i32,
                );
            },
            AttributeValueType::Int => unsafe {
                gl.vertex_attrib_pointer_i32(
                    index as u32,
                    attribute.num_elements as i32,
                    attribute.element_type.to_gl(),
                    std::mem::size_of::<V>() as i32,
                    attribute.offset as i32,
                );
            },
        }
    }
}
