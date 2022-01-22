use std::rc::Rc;

use nalgebra::{Point2, Vector2};
use thiserror::Error;

use glow::HasContext;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::ImageBitmap;

use crate::FetchError;

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
    #[error("new texture error: {0}")]
    NewTexture(#[from] NewTextureError),

    #[error("fetch error: {0}")]
    Fetch(#[from] FetchError),

    #[error("failed to create image bitmap: {0:?}")]
    CreateImageBitmap(JsValue),

    #[error("failed to await image bitmap: {0:?}")]
    AwaitCreateImageBitmap(JsValue),

    #[error("failed to retrieve image bitmap length: {0:?}")]
    MappedDataLength(JsValue),

    #[error("failed to map image bitmap data: {0:?}")]
    MapData(JsValue),

    #[error("failed to await image bitmap data: {0:?}")]
    AwaitMapData(JsValue),
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
    RgbaF16,
    RgbaF32,
    RgF16,
    Depth,
    // TODO: Texture value type is incomplete
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
            texture.generate_mipmap();
        }

        Ok(texture)
    }

    pub async fn from_image_bitmap(
        gl: Rc<Context>,
        image_bitmap: ImageBitmap,
        params: TextureParams,
    ) -> Result<Self, LoadTextureError> {
        assert!(params.value_type == TextureValueType::RgbaU8);

        let size = Vector2::new(image_bitmap.width(), image_bitmap.height());
        let texture = Self::new_impl(gl.clone(), size, params.clone())?;

        unsafe {
            // FIXME: Not sure if ImageBitmap applies color space conversion here.
            gl.tex_image_2d_with_image_bitmap(
                glow::TEXTURE_2D,
                0,
                params.value_type.internal_format_gl(),
                params.value_type.format_gl(),
                params.value_type.type_gl(),
                &image_bitmap,
            );
        }

        if texture.params.min_filter.uses_mipmap() {
            texture.generate_mipmap();
        }

        Ok(texture)
    }

    pub async fn load(
        gl: Rc<Context>,
        path: &str,
        params: TextureParams,
    ) -> Result<Self, LoadTextureError> {
        let blob = crate::fetch_blob(path).await?;
        let image_bitmap: ImageBitmap = {
            let promise = web_sys::window()
                .unwrap()
                .create_image_bitmap_with_blob(&blob)
                .map_err(LoadTextureError::CreateImageBitmap)?;
            let value = JsFuture::from(promise)
                .await
                .map_err(LoadTextureError::AwaitCreateImageBitmap)?;
            assert!(value.is_instance_of::<ImageBitmap>());
            value.dyn_into().unwrap()
        };

        Self::from_image_bitmap(gl, image_bitmap, params).await
    }

    pub async fn from_data(
        gl: Rc<Context>,
        data: &mut [u8],
        params: TextureParams,
    ) -> Result<Self, LoadTextureError> {
        let image_bitmap: ImageBitmap = {
            // TODO: Why does this need &mut [u8]?
            let promise = web_sys::window()
                .unwrap()
                .create_image_bitmap_with_u8_array(data)
                .map_err(LoadTextureError::CreateImageBitmap)?;
            let value = JsFuture::from(promise)
                .await
                .map_err(LoadTextureError::AwaitCreateImageBitmap)?;
            assert!(value.is_instance_of::<ImageBitmap>());
            value.dyn_into().unwrap()
        };

        Self::from_image_bitmap(gl, image_bitmap, params).await
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
            self.generate_mipmap();
        }
    }

    pub fn generate_mipmap(&self) {
        assert!(self.params.min_filter.uses_mipmap());

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.id));
            self.gl.generate_mipmap(glow::TEXTURE_2D);
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

impl TextureParams {
    pub fn mipmapped() -> Self {
        Self {
            value_type: TextureValueType::RgbaU8,
            min_filter: TextureMinFilter::LinearMipmapLinear,
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
            RgbaF16 => glow::RGBA16F as i32,
            RgbaF32 => glow::RGBA32F as i32,
            RgF16 => glow::RG16F as i32,
            Depth => glow::DEPTH_COMPONENT24 as i32,
        }
    }

    pub fn format_gl(self) -> u32 {
        use TextureValueType::*;

        match self {
            RgbaU8 => glow::RGBA,
            RgbaF16 => glow::RGBA,
            RgbaF32 => glow::RGBA,
            RgF16 => glow::RG,
            Depth => glow::DEPTH_COMPONENT,
        }
    }

    pub fn type_gl(self) -> u32 {
        use TextureValueType::*;

        match self {
            RgbaU8 => glow::UNSIGNED_BYTE,
            RgbaF16 => glow::HALF_FLOAT,
            RgbaF32 => glow::FLOAT,
            RgF16 => glow::HALF_FLOAT,
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
