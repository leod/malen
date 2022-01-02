use std::rc::Rc;

use glow::HasContext;
use nalgebra::Vector2;
use thiserror::Error;

use super::Context;

#[derive(Error, Debug)]
pub enum NewTextureError {
    #[error("GL error: {0}")]
    OpenGL(#[from] super::Error),

    #[error("image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("texture width too large: requested {0}, but max size is {1}")]
    WidthTooLarge(u32, u32),

    #[error("texture height too large: requestd {0}, but max size is {1}")]
    HeightTooLarge(u32, u32),
}

#[derive(Debug, Clone)]
pub struct TextureParams {
    pub value_type: TextureValueType,
    pub min_filter: TextureMinFilter,
    pub mag_filter: TextureMagFilter,
    pub wrap_vertical: TextureWrap,
    pub wrap_horizontal: TextureWrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureValueType {
    UnsignedByte,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureMinFilter {
    Linear,
    Nearest,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureMagFilter {
    Linear,
    Nearest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureWrap {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

pub struct Texture {
    gl: Rc<Context>,
    pub(crate) texture: <glow::Context as HasContext>::Texture,
    size: Vector2<u32>,
}

impl Texture {
    pub fn new(
        gl: Rc<Context>,
        size: Vector2<u32>,
        params: TextureParams,
    ) -> Result<Self, NewTextureError> {
        assert!(
            !params.min_filter.uses_mipmap(),
            "Empty textures do not have a mipmap"
        );

        let texture = Self::new_impl(gl.clone(), size, &params)?;

        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                size.x as i32,
                size.y as i32,
                0,
                glow::RGBA,
                params.value_type.to_gl(),
                None,
            );
        }

        Ok(texture)
    }

    pub fn from_rgba(
        gl: Rc<Context>,
        rgba: &[u8],
        size: Vector2<u32>,
        params: TextureParams,
    ) -> Result<Self, NewTextureError> {
        assert!(rgba.len() as u32 == size.x * size.y * 4);

        let texture = Self::new_impl(gl.clone(), size, &params)?;

        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                size.x as i32,
                size.y as i32,
                0,
                glow::RGBA,
                params.value_type.to_gl(),
                Some(rgba),
            );
        }

        if params.min_filter.uses_mipmap() {
            unsafe {
                gl.generate_mipmap(glow::TEXTURE_2D);
            }
        }

        Ok(texture)
    }

    pub fn from_encoded_bytes(
        gl: Rc<Context>,
        encoded_bytes: &[u8],
        params: TextureParams,
    ) -> Result<Self, NewTextureError> {
        let image = image::load_from_memory(encoded_bytes)?.to_rgba8();
        let size = Vector2::new(image.width(), image.height());

        Self::from_rgba(gl, image.into_raw().as_slice(), size, params)
    }

    fn new_impl(
        gl: Rc<Context>,
        size: Vector2<u32>,
        params: &TextureParams,
    ) -> Result<Self, NewTextureError> {
        assert!(size.x > 0, "Texture width must be positive");
        assert!(size.y > 0, "Texture height must be positive");
        {
            let max_size = unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) } as u32;
            if size.x > max_size {
                return Err(NewTextureError::WidthTooLarge(size.x, max_size));
            }
            if size.y > max_size {
                return Err(NewTextureError::HeightTooLarge(size.y, max_size));
            }
        }

        // TODO:
        // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#dont_assume_you_can_render_into_float_textures

        let texture = unsafe { gl.create_texture() }.map_err(super::Error::Glow)?;

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        }

        set_texture_params(&*gl, params);

        Ok(Self { gl, texture, size })
    }

    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}

impl Default for TextureParams {
    fn default() -> Self {
        Self {
            value_type: TextureValueType::UnsignedByte,
            min_filter: TextureMinFilter::Linear,
            mag_filter: TextureMagFilter::Linear,
            wrap_vertical: TextureWrap::Repeat,
            wrap_horizontal: TextureWrap::Repeat,
        }
    }
}

impl TextureValueType {
    pub fn to_gl(self) -> u32 {
        use TextureValueType::*;

        match self {
            UnsignedByte => glow::UNSIGNED_BYTE,
            Float => glow::FLOAT,
        }
    }
}

impl TextureMinFilter {
    pub fn to_gl(self) -> u32 {
        use TextureMinFilter::*;

        match self {
            Linear => glow::LINEAR,
            Nearest => glow::NEAREST,
            NearestMipmapNearest => glow::NEAREST_MIPMAP_NEAREST,
            LinearMipmapNearest => glow::LINEAR_MIPMAP_NEAREST,
            NearestMipmapLinear => glow::NEAREST_MIPMAP_LINEAR,
            LinearMipmapLinear => glow::LINEAR_MIPMAP_LINEAR,
        }
    }

    pub fn uses_mipmap(self) -> bool {
        use TextureMinFilter::*;

        self != Linear && self != Nearest
    }
}

impl TextureMagFilter {
    pub fn to_gl(self) -> u32 {
        use TextureMagFilter::*;

        match self {
            Linear => glow::LINEAR,
            Nearest => glow::NEAREST,
        }
    }
}

impl TextureWrap {
    pub fn to_gl(self) -> u32 {
        use TextureWrap::*;

        match self {
            Repeat => glow::REPEAT,
            ClampToEdge => glow::CLAMP_TO_EDGE,
            MirroredRepeat => glow::MIRRORED_REPEAT,
        }
    }
}

fn set_texture_params(gl: &Context, params: &TextureParams) {
    unsafe {
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            params.min_filter.to_gl() as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            params.mag_filter.to_gl() as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            params.wrap_horizontal.to_gl() as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            params.wrap_vertical.to_gl() as i32,
        );
    }
}
