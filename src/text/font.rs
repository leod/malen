use std::{collections::HashMap, rc::Rc};

use fontdue::{
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
    FontSettings,
};
use nalgebra::{Point3, Vector2};
use thiserror::Error;

use super::atlas::Atlas;
use crate::{
    geometry::{ColorSprite, ColorSpriteBatch},
    gl::{self, Blend, DrawParams, NewTextureError, Texture, UniformBuffer},
    pass::ColorSpritePass,
    util, Color4, Context, MatricesBlock, Rect,
};

#[derive(Error, Debug)]
pub enum NewFontError {
    #[error("new texture error: {0}")]
    NewTexture(#[from] NewTextureError),

    #[error("fontdue error: {0}")]
    Fontdue(&'static str),
}

#[derive(Error, Debug)]
pub enum WriteTextError {
    #[error("OpenGL error: {0}")]
    OpenGL(#[from] gl::Error),

    #[error("texture error: {0}")]
    Texture(#[from] NewTextureError),

    #[error("glyph too large: glyph {0:?} of size {1} did not fit into empty atlas of size {2}")]
    GlyphTooLarge(GlyphRasterConfig, Vector2<u32>, u32),
}

pub struct TextBatch {
    by_atlas: Vec<ColorSpriteBatch>,
}

#[derive(Debug, Clone)]
struct GlyphLoc {
    atlas_index: usize,
    tex_rect: Rect,
}

pub struct Font {
    gl: Rc<gl::Context>,

    font: fontdue::Font,
    layout: Layout,

    atlases: Vec<Atlas>,
    glyph_locs: HashMap<GlyphRasterConfig, GlyphLoc>,
    bitmap_buffer: Vec<u8>,

    color_sprite_pass: Rc<ColorSpritePass>,
}

const MAX_ATLAS_SIZE: u32 = 2048;

impl Font {
    pub fn from_bytes<Data>(
        context: &Context,
        data: &[u8],
        scale: f32,
    ) -> Result<Self, NewFontError> {
        let settings = FontSettings {
            scale,
            ..Default::default()
        };

        let font = fontdue::Font::from_bytes(data, settings).map_err(NewFontError::Fontdue)?;

        let atlas_size = Texture::max_size(&*context.gl()).min(MAX_ATLAS_SIZE);
        let atlas = Atlas::new(context.gl(), atlas_size, atlas_size)?;
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Ok(Font {
            gl: context.gl(),
            font,
            layout,
            atlases: vec![atlas],
            glyph_locs: HashMap::new(),
            bitmap_buffer: Vec::new(),
            color_sprite_pass: context.color_sprite_pass(),
        })
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.gl.clone()
    }

    pub fn text_size(&mut self, size: f32, text: &str) -> Vector2<f32> {
        let settings = LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: None,
            ..Default::default()
        };
        self.layout.reset(&settings);

        self.layout
            .append(&[&self.font], &TextStyle::new(text, size, 0));

        self.layout
            .glyphs()
            .last()
            .map_or(Vector2::zeros(), |glyph_pos| {
                Vector2::new(
                    glyph_pos.x + glyph_pos.width as f32,
                    glyph_pos.y + glyph_pos.height as f32,
                )
            })
    }

    pub fn write(
        &mut self,
        pos: Point3<f32>,
        size: f32,
        color: Color4,
        text: &str,
        batch: &mut TextBatch,
    ) -> Result<Vector2<f32>, WriteTextError> {
        let dpr = util::device_pixel_ratio() as f32;

        self.layout.reset(&LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: None,
            ..Default::default()
        });
        self.layout
            .append(&[&self.font], &TextStyle::new(text, size * dpr, 0));

        let mut last_offset = Vector2::zeros();

        for glyph_pos in self.layout.glyphs() {
            // Ignore empty glyphs (e.g. space).
            if glyph_pos.width == 0 || glyph_pos.height == 0 {
                continue;
            }

            let glyph_loc = Self::get_glyph_loc(
                self.gl.clone(),
                &self.font,
                &mut self.atlases,
                &mut self.glyph_locs,
                &mut self.bitmap_buffer,
                &glyph_pos.key,
            )?;

            while batch.by_atlas.len() < glyph_loc.atlas_index + 1 {
                batch.by_atlas.push(ColorSpriteBatch::new(self.gl.clone())?);
            }

            let offset = Vector2::new(
                glyph_pos.x + glyph_pos.width as f32 / 2.0,
                glyph_pos.y + glyph_pos.height as f32 / 2.0,
            ) / dpr;

            let rect = Rect {
                center: pos.xy() + offset,
                size: Vector2::new(glyph_pos.width as f32, glyph_pos.height as f32) / dpr,
            };

            batch.by_atlas[glyph_loc.atlas_index].push(ColorSprite {
                rect,
                z: pos.z,
                tex_rect: glyph_loc.tex_rect,
                color,
            });

            last_offset = offset;
        }

        Ok(last_offset)
    }

    pub fn draw(
        &mut self,
        matrices_buffer: &UniformBuffer<MatricesBlock>,
        batch: &mut TextBatch,
    ) -> Result<(), gl::Error> {
        for (atlas_batch, atlas) in batch.by_atlas.iter_mut().zip(&self.atlases) {
            self.color_sprite_pass.draw(
                matrices_buffer,
                atlas.texture(),
                atlas_batch.draw_unit(),
                &DrawParams {
                    blend: Some(Blend::default()),
                    ..DrawParams::default()
                },
            )?;
        }

        Ok(())
    }

    fn get_glyph_loc(
        gl: Rc<gl::Context>,
        font: &fontdue::Font,
        atlases: &mut Vec<Atlas>,
        glyph_locs: &mut HashMap<GlyphRasterConfig, GlyphLoc>,
        bitmap_buffer: &mut Vec<u8>,
        glyph_key: &GlyphRasterConfig,
    ) -> Result<GlyphLoc, WriteTextError> {
        if let Some(glyph_loc) = glyph_locs.get(glyph_key) {
            Ok(glyph_loc.clone())
        } else {
            let (metrics, alpha_bitmap) =
                font.rasterize_indexed(glyph_key.glyph_index, glyph_key.px);
            let glyph_size = Vector2::new(
                u32::try_from(metrics.width).unwrap(),
                u32::try_from(metrics.height).unwrap(),
            );

            Self::alpha_to_rgba(&alpha_bitmap, bitmap_buffer);

            for (atlas_index, atlas) in atlases.iter_mut().enumerate() {
                let tex_rect = atlas.insert(bitmap_buffer.as_slice(), glyph_size);

                if let Some(tex_rect) = tex_rect {
                    return Ok(GlyphLoc {
                        atlas_index,
                        tex_rect,
                    });
                }
            }

            let atlas_size = Texture::max_size(&*gl).min(MAX_ATLAS_SIZE);
            let mut atlas = Atlas::new(gl.clone(), atlas_size, atlas_size)?;

            let tex_rect = atlas.insert(bitmap_buffer.as_slice(), glyph_size);

            atlases.push(atlas);

            if let Some(tex_rect) = tex_rect {
                Ok(GlyphLoc {
                    atlas_index: atlases.len() - 1,
                    tex_rect,
                })
            } else {
                Err(WriteTextError::GlyphTooLarge(
                    glyph_key.clone(),
                    glyph_size,
                    atlas_size,
                ))
            }
        }
    }

    fn alpha_to_rgba(bitmap: &[u8], output: &mut Vec<u8>) {
        output.clear();
        for v in bitmap {
            let v = *v;
            output.extend_from_slice(&[255, 255, 255, v]);
        }
    }
}
