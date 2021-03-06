use std::{collections::HashMap, ops::Deref};

use fontdue::{
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
    FontSettings,
};
use golem::blend::{BlendEquation, BlendFactor, BlendFunction, BlendMode, BlendOperation};
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use crate::{
    draw::{
        text::packer::ShelfPacker, DrawUnit, Quad, TexColPass, TexColVertex, Texture, TriBatch,
    },
    AaRect, Canvas, Color4, Error,
};

pub type TextBatch = TriBatch<TexColVertex>;

struct Glyph {
    uv_rect: AaRect,
}

pub struct Font {
    font: fontdue::Font,
    layout: Layout,

    packer: ShelfPacker,
    cache: HashMap<GlyphRasterConfig, Glyph>,

    pass: TexColPass,

    bitmap_buffer: Vec<u8>,
}

const ATLAS_WIDTH: usize = 512;
const ATLAS_HEIGHT: usize = 256;

impl Font {
    pub fn from_bytes<Data>(ctx: &Canvas, data: Data, scale: f32) -> Result<Self, Error>
    where
        Data: Deref<Target = [u8]>,
    {
        let settings = FontSettings {
            scale,
            ..Default::default()
        };

        let font =
            fontdue::Font::from_bytes(data, settings).map_err(|msg| Error::Font(msg.into()))?;

        let packer = ShelfPacker::new(ctx, ATLAS_WIDTH, ATLAS_HEIGHT)?;
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        let pass = TexColPass::new(ctx)?;

        Ok(Font {
            font,
            packer,
            layout,
            cache: HashMap::new(),
            pass,
            bitmap_buffer: Vec::new(),
        })
    }

    pub fn write(
        &mut self,
        size: f32,
        pos: Point3<f32>,
        color: Color4,
        text: &str,
        batch: &mut TextBatch,
    ) {
        let settings = LayoutSettings {
            x: pos.x,
            y: pos.y,
            max_width: None,
            ..Default::default()
        };
        self.layout.reset(&settings);

        self.layout
            .append(&[&self.font], &TextStyle::new(text, size, 0));

        for &glyph_pos in self.layout.glyphs() {
            // Ignore empty glyphs (e.g. space).
            if glyph_pos.width == 0 || glyph_pos.height == 0 {
                continue;
            }

            let (font, packer, bitmap_buffer) =
                (&self.font, &mut self.packer, &mut self.bitmap_buffer);

            let glyph = self.cache.entry(glyph_pos.key).or_insert_with(|| {
                let (metrics, alpha_bitmap) = font.rasterize(glyph_pos.key.c, size);

                Self::alpha_to_rgba(&alpha_bitmap, bitmap_buffer);

                let uv_rect = packer
                    .insert(bitmap_buffer.as_slice(), metrics.width, metrics.height)
                    .unwrap(); // TODO: unwrap in atlas insert

                Glyph { uv_rect }
            });

            let rect_center = Point2::new(
                glyph_pos.x + glyph_pos.width as f32 / 2.0,
                glyph_pos.y + glyph_pos.height as f32 / 2.0,
            );
            let rect_size = Vector2::new(glyph_pos.width as f32, glyph_pos.height as f32);

            batch.push_quad(
                &Quad::axis_aligned(rect_center, rect_size),
                pos.z,
                glyph.uv_rect,
                color,
            );
        }
    }

    pub fn draw(
        &mut self,
        ctx: &Canvas,
        transform: &Matrix3<f32>,
        draw_unit: &DrawUnit<TexColVertex>,
    ) -> Result<(), Error> {
        ctx.golem_ctx().set_blend_mode(Some(BlendMode {
            equation: BlendEquation::Same(BlendOperation::Add),
            function: BlendFunction::Same {
                source: BlendFactor::One,
                destination: BlendFactor::One,
            },
            ..Default::default()
        }));

        self.pass
            .draw(transform, self.packer.texture(), draw_unit)?;

        ctx.golem_ctx().set_blend_mode(None);

        Ok(())
    }

    pub fn texture(&self) -> &Texture {
        self.packer.texture()
    }

    fn alpha_to_rgba(bitmap: &[u8], output: &mut Vec<u8>) {
        output.clear();
        for v in bitmap {
            let v = *v;
            output.extend_from_slice(&[v, v, v, v]);
        }
    }
}
