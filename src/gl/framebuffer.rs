use std::rc::Rc;

use glow::{HasContext, PixelPackData};
use half::f16;
use thiserror::Error;

use crate::gl::TextureValueType;

use super::{Context, NewTextureError, Texture};

#[derive(Error, Debug)]
pub enum NewFramebufferError {
    #[error("GL error: {0}")]
    OpenGL(#[from] super::Error),

    #[error("too many color attachments: requested {0}, but max count is {1}")]
    TooManyColorAttachments(usize, u32),

    #[error("texture error: {0}")]
    Texture(#[from] NewTextureError),
}

pub struct Framebuffer {
    gl: Rc<Context>,
    textures: Vec<Rc<Texture>>,
    id: glow::Framebuffer,
    attachments: Vec<u32>,
}

impl Framebuffer {
    pub fn max_color_attachments(gl: &Context) -> u32 {
        let max_color_attachments = unsafe { gl.get_parameter_i32(glow::MAX_COLOR_ATTACHMENTS) };
        max_color_attachments as u32
    }

    pub fn from_textures(
        gl: Rc<Context>,
        textures: Vec<Texture>,
    ) -> Result<Self, NewFramebufferError> {
        Self::new(gl.clone(), textures.into_iter().map(Rc::new).collect())
    }

    pub fn new(gl: Rc<Context>, textures: Vec<Rc<Texture>>) -> Result<Self, NewFramebufferError> {
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
        }

        let mut draw_buffers = Vec::new();
        let mut attachments = Vec::new();
        for (location, texture) in textures
            .iter()
            .filter(|t| !t.params().value_type.is_depth())
            .enumerate()
        {
            let attachment = glow::COLOR_ATTACHMENT0 + location as u32;
            draw_buffers.push(attachment);
            attachments.push(attachment);

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
            attachments.push(attachment);

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

        unsafe {
            gl.draw_buffers(&draw_buffers);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        Ok(Framebuffer {
            gl,
            textures,
            id,
            attachments,
        })
    }

    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn textures(&self) -> &[Rc<Texture>] {
        &self.textures
    }

    pub fn id(&self) -> glow::Framebuffer {
        self.id
    }

    pub fn read_pixel_row_f16(&self, location: usize, y: u32) -> Vec<f16> {
        let texture = &self.textures[location];
        let attachment = glow::COLOR_ATTACHMENT0 + location as u32;

        // TODO
        assert!(texture.params().value_type == TextureValueType::RgF16);

        let mut data: Vec<f16> = vec![f16::from_f32(0.0); 2 * texture.size().x as usize];

        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.id));
            self.gl.read_buffer(attachment);
            self.gl.read_pixels(
                0,
                i32::try_from(y).unwrap(),
                i32::try_from(texture.size().x).unwrap(),
                1,
                texture.params().value_type.format_gl(),
                texture.params().value_type.type_gl(),
                PixelPackData::Slice(bytemuck::cast_slice_mut(&mut data)),
            );
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        data
    }

    pub fn invalidate(&self) {
        let gl = self.gl();

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.id));
            gl.invalidate_framebuffer(glow::FRAMEBUFFER, &self.attachments);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.id);
        }
    }
}
