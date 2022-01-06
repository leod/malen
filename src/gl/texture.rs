use std::rc::Rc;

use glow::HasContext;
use nalgebra::{Point2, Vector2};
use thiserror::Error;

use super::Context;

#[derive(Error, Debug)]
pub enum NewTextureError {
    #[error("GL error: {0}")]
    OpenGL(#[from] super::Error),

    #[error("texture too large: requested {0}, but max size is {1}")]
    TooLarge(u32, u32),
}

#[derive(Error, Debug)]
pub enum LoadTextureError {
    #[error("texture error: {0}")]
    NewTexture(#[from] NewTextureError),

    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
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
    RgbaU8,
    RgbaF32,
    Depth,
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
    id: glow::Texture,
    size: Vector2<u32>,
    params: TextureParams,
}

impl Texture {
    pub fn max_size(gl: &Context) -> u32 {
        let max_size = unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) };
        max_size as u32
    }

    pub fn new(
        gl: Rc<Context>,
        size: Vector2<u32>,
        params: TextureParams,
    ) -> Result<Self, NewTextureError> {
        assert!(
            !params.min_filter.uses_mipmap(),
            "Empty textures do not have a mipmap"
        );

        let texture = Self::new_impl(gl.clone(), size, params.clone())?;

        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                params.value_type.internal_format_gl(),
                i32::try_from(size.x).unwrap(),
                i32::try_from(size.y).unwrap(),
                0,
                params.value_type.format_gl(),
                params.value_type.type_gl(),
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
        assert!(params.value_type == TextureValueType::RgbaU8);

        let texture = Self::new_impl(gl.clone(), size, params.clone())?;

        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                params.value_type.internal_format_gl(),
                i32::try_from(size.x).unwrap(),
                i32::try_from(size.y).unwrap(),
                0,
                params.value_type.format_gl(),
                params.value_type.type_gl(),
                Some(rgba),
            );
        }

        if texture.params.min_filter.uses_mipmap() {
            unsafe {
                gl.generate_mipmap(glow::TEXTURE_2D);
            }
        }

        Ok(texture)
    }

    pub fn load(
        gl: Rc<Context>,
        encoded_bytes: &[u8],
        params: TextureParams,
    ) -> Result<Self, LoadTextureError> {
        let image = image::load_from_memory(encoded_bytes)?.to_rgba8();
        let size = Vector2::new(image.width(), image.height());

        Ok(Self::from_rgba(
            gl,
            image.into_raw().as_slice(),
            size,
            params,
        )?)
    }

    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn id(&self) -> glow::Texture {
        self.id
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }

    pub fn params(&self) -> &TextureParams {
        &self.params
    }

    pub fn set_sub_image(&self, pos: Point2<u32>, size: Vector2<u32>, data: &[u8]) {
        assert!(pos.x + size.x <= self.size.x);
        assert!(pos.y + size.y <= self.size.y);
        assert!(self.params.value_type == TextureValueType::RgbaU8);

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.id));
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                i32::try_from(pos.x).unwrap(),
                i32::try_from(pos.y).unwrap(),
                i32::try_from(size.x).unwrap(),
                i32::try_from(size.y).unwrap(),
                self.params.value_type.format_gl(),
                self.params.value_type.type_gl(),
                glow::PixelUnpackData::Slice(data),
            );
        }

        if self.params.min_filter.uses_mipmap() {
            unsafe {
                self.gl.generate_mipmap(glow::TEXTURE_2D);
            }
        }
    }

    fn new_impl(
        gl: Rc<Context>,
        size: Vector2<u32>,
        params: TextureParams,
    ) -> Result<Self, NewTextureError> {
        assert!(size.x > 0, "Texture width must be positive");
        assert!(size.y > 0, "Texture height must be positive");

        if size.x > Self::max_size(&*gl) {
            return Err(NewTextureError::TooLarge(size.x, Self::max_size(&*gl)));
        }
        if size.y > Self::max_size(&*gl) {
            return Err(NewTextureError::TooLarge(size.y, Self::max_size(&*gl)));
        }

        // TODO:
        // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#dont_assume_you_can_render_into_float_textures

        let id = unsafe { gl.create_texture() }.map_err(super::Error::Glow)?;

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(id));
        }

        set_texture_params(&*gl, &params);

        Ok(Self {
            gl,
            id,
            size,
            params,
        })
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.id);
        }
    }
}

impl Default for TextureParams {
    fn default() -> Self {
        Self {
            value_type: TextureValueType::RgbaU8,
            min_filter: TextureMinFilter::Linear,
            mag_filter: TextureMagFilter::Linear,
            wrap_vertical: TextureWrap::Repeat,
            wrap_horizontal: TextureWrap::Repeat,
        }
    }
}

impl TextureValueType {
    pub fn internal_format_gl(self) -> i32 {
        use TextureValueType::*;

        match self {
            RgbaU8 => glow::RGBA8 as i32,
            RgbaF32 => glow::RGBA32F as i32,
            Depth => glow::DEPTH_COMPONENT24 as i32,
        }
    }

    pub fn format_gl(self) -> u32 {
        use TextureValueType::*;

        match self {
            RgbaU8 => glow::RGBA,
            RgbaF32 => glow::RGBA,
            Depth => glow::DEPTH_COMPONENT,
        }
    }

    pub fn type_gl(self) -> u32 {
        use TextureValueType::*;

        match self {
            RgbaU8 => glow::UNSIGNED_BYTE,
            RgbaF32 => glow::FLOAT,
            Depth => glow::UNSIGNED_INT,
        }
    }

    pub fn is_depth(self) -> bool {
        self == TextureValueType::Depth
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
