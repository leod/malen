use std::{marker::PhantomData, rc::Rc};

use crevice::std140::AsStd140;
use glow::HasContext;

use super::{Context, Error};

pub struct UniformBuffer<U> {
    gl: Rc<Context>,
    pub(super) buffer: <glow::Context as HasContext>::Buffer,
    _phantom: PhantomData<U>,
}

impl<U> UniformBuffer<U>
where
    U: AsStd140,
{
    pub fn new(gl: Rc<Context>, uniform: U) -> Result<Self, Error> {
        let buffer = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;
        let uniform_buffer = UniformBuffer {
            gl,
            buffer,
            _phantom: PhantomData,
        };

        uniform_buffer.set_data(uniform);

        Ok(uniform_buffer)
    }

    pub fn set_data(&self, data: U) {
        let data_std140 = data.as_std140();
        let data_u8 = bytemuck::bytes_of(&data_std140);

        unsafe {
            self.gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.buffer));
            self.gl
                .buffer_data_u8_slice(glow::UNIFORM_BUFFER, data_u8, glow::STREAM_DRAW);
        }
    }
}

impl<U> UniformBuffer<U> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }
}

impl<U> Drop for UniformBuffer<U> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
        }
    }
}
