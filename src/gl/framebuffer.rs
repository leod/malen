use std::rc::Rc;

use glow::HasContext;
use thiserror::Error;

use super::{Context, Texture};

#[derive(Error, Debug)]
pub enum NewFramebufferError {
    #[error("GL error: {0}")]
    OpenGL(#[from] super::Error),

    #[error("too many color attachments: requested {0}, but max count is {1}")]
    TooManyColorAttachments(usize, u32),
}

pub struct Framebuffer {
    gl: Rc<Context>,
    textures: Vec<Texture>,
    id: glow::Framebuffer,
}

impl Framebuffer {
    pub fn max_color_attachments(gl: &Context) -> u32 {
        let max_color_attachments = unsafe { gl.get_parameter_i32(glow::MAX_COLOR_ATTACHMENTS) };
        max_color_attachments as u32
    }

    pub fn new(gl: Rc<Context>, textures: Vec<Texture>) -> Result<Self, NewFramebufferError> {
        let num_color = textures
            .iter()
            .filter(|t| !t.params().value_type.is_depth())
            .count();
        let num_depth = textures.len() - num_color;

        assert!(
            num_color + num_depth > 0,
            "Must have at least one attachment"
        );
        assert!(num_depth <= 1, "Can have at most one depth attachment");
        assert!(textures
            .iter()
            .all(|t| t.size() == textures.first().unwrap().size()));

        if num_color > Self::max_color_attachments(&*gl) as usize {
            return Err(NewFramebufferError::TooManyColorAttachments(
                num_color,
                Self::max_color_attachments(&*gl),
            ));
        }

        let id = unsafe { gl.create_framebuffer() }.map_err(super::Error::Glow)?;

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(id));
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        for (i, texture) in textures
            .iter()
            .filter(|t| !t.params().value_type.is_depth())
            .enumerate()
        {
            let attachment = glow::COLOR_ATTACHMENT0 + i as u32;

            unsafe {
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    attachment,
                    glow::TEXTURE_2D,
                    Some(texture.id()),
                    0,
                );
            }
        }

        for texture in textures.iter().filter(|t| t.params().value_type.is_depth()) {
            let attachment = glow::DEPTH_ATTACHMENT;

            unsafe {
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    attachment,
                    glow::TEXTURE_2D,
                    Some(texture.id()),
                    0,
                );
            }
        }

        Ok(Framebuffer { gl, textures, id })
    }

    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }

    pub fn id(&self) -> glow::Framebuffer {
        self.id
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.id);
        }
    }
}
