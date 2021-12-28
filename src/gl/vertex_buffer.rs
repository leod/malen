use std::{marker::PhantomData, rc::Rc};

use glow::HasContext;

use crate::{
    gl::{self, ElementType, Vertex},
    Error,
};

pub struct VertexBuffer<V> {
    gl: Rc<gl::Context>,
    vao: <glow::Context as HasContext>::VertexArray,
    buffer: <glow::Context as HasContext>::Buffer,
    len: usize,
    _phantom: PhantomData<V>,
}

impl<V: Vertex> VertexBuffer<V> {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, Error> {
        let vao = unsafe { gl.create_vertex_array() }.map_err(Error::Glow)?;
        let buffer = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
        }

        set_vertex_attribs::<V>(gl);

        Ok(Self {
            gl,
            vao,
            buffer,
            len: 0,
            _phantom: PhantomData,
        })
    }

    pub fn set_data(&mut self, data: &[V]) {
        let data_u8 = bytemuck::cast_slice(data);

        // TODO: Prevent implicit synchronization somehow.
        // https://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer));
            self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data_u8, glow::STREAM_DRAW);
        }
    }
}

fn set_vertex_attribs<V: Vertex>(gl: Rc<gl::Context>) {
    for (index, attribute) in V::attributes().iter().enumerate() {
        assert!(
            attribute.offset + attribute.num_elements * attribute.element_type.size()
                <= std::mem::size_of::<V>()
        );

        unsafe {
            gl.enable_vertex_attrib_array(index as u32);
        }

        match attribute.element_type {
            ElementType::Float => unsafe {
                gl.vertex_attrib_pointer_f32(
                    index as u32,
                    attribute.num_elements as i32,
                    attribute.element_type.to_gl(),
                    false,
                    std::mem::size_of::<V>() as i32,
                    attribute.offset as i32,
                );
            },
            ElementType::Int => unsafe {
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

impl<V> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.buffer);
        }
    }
}
