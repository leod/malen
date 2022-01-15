use std::{marker::PhantomData, rc::Rc};

use crevice::std140::AsStd140;
use glow::HasContext;

use super::{Context, Error};

pub struct Uniform<U> {
    gl: Rc<Context>,
    id: <glow::Context as HasContext>::Buffer,
    _phantom: PhantomData<U>,
}

impl<U> Uniform<U>
where
    U: AsStd140,
{
    pub fn new(gl: Rc<Context>, uniform: U) -> Result<Self, Error> {
        let id = unsafe { gl.create_buffer() }.map_err(Error::Glow)?;
        let uniform_buffer = Uniform {
            gl,
            id,
            _phantom: PhantomData,
        };

        uniform_buffer.set_data(uniform);

        Ok(uniform_buffer)
    }

    pub fn set_data(&self, data: U) {
        let data_std140 = data.as_std140();
        let data_u8 = bytemuck::bytes_of(&data_std140);

        unsafe {
            self.gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.id));
            self.gl
                .buffer_data_u8_slice(glow::UNIFORM_BUFFER, data_u8, glow::STREAM_DRAW);
        }
    }
}

impl<U> Uniform<U> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn id(&self) -> glow::Buffer {
        self.id
    }
}

impl<U> Drop for Uniform<U> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.id);
        }
    }
}
