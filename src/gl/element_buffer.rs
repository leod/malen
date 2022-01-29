use std::{cell::Cell, marker::PhantomData, rc::Rc};

use bytemuck::{Pod, Zeroable};
use glow::HasContext;

use super::{Context, Error};

pub trait Element: Pod + Zeroable {
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

pub struct ElementBuffer<E = u32> {
    gl: Rc<Context>,
    id: glow::Buffer,
    len: Cell<usize>,
    capacity: Cell<usize>,
    _phantom: PhantomData<E>,
}

impl<E> ElementBuffer<E>
where
    E: Element,
{
    pub fn new(gl: Rc<Context>) -> Result<Self, Error> {
        let id = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;

        Ok(Self {
            gl,
            id,
            len: Cell::new(0),
            capacity: Cell::new(0),
            _phantom: PhantomData,
        })
    }

    pub fn new_static(gl: Rc<Context>, data: &[E]) -> Result<Self, Error> {
        let element_buffer = Self::new(gl)?;
        element_buffer.set_data_with_usage(data, glow::STATIC_DRAW);

        Ok(element_buffer)
    }

    pub fn set(&self, data: &[E]) {
        // TODO: Prevent implicit synchronization somehow.
        // https://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf

        self.set_data_with_usage(data, glow::STREAM_DRAW);
    }

    fn set_data_with_usage(&self, data: &[E], usage: u32) {
        let data_u8 = bytemuck::cast_slice(data);

        unsafe {
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.id));
            self.gl
                .buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data_u8, usage);
        }

        self.len.set(data.len());
    }
}

impl<E> ElementBuffer<E> {
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

impl<E> Drop for ElementBuffer<E> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.id);
        }
    }
}
