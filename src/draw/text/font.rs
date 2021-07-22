use std::{collections::HashMap, ops::Deref};

use fontdue::{
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
    FontSettings,
};
use golem::blend::BlendMode;
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use crate::{
    draw::{
        text::packer::ShelfPacker, DrawUnit, Quad, TexColPass, TexColVertex, Texture, TriBatch,
    },
    AxisRect, Canvas, Color4, Error,
};

pub type TextBatch = TriBatch<TexColVertex>;

struct Glyph {
    uv_rect: AxisRect,
}

pub struct Font {
    font: fontdue::Font,
    layout: Layout,

    packer: ShelfPacker,
    cache: HashMap<GlyphRasterConfig, Glyph>,

    device_pixel_ratio: f32,
    pass: TexColPass,

    bitmap_buffer: Vec<u8>,
}

const ATLAS_WIDTH: usize = 2048;
const ATLAS_HEIGHT: usize = 2048;

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
            device_pixel_ratio: 1.0,
            pass,
            bitmap_buffer: Vec::new(),
        })
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
        size: f32,
        pos: Point3<f32>,
        color: Color4,
        text: &str,
        batch: &mut TextBatch,
    ) -> Vector2<f32> {
        let settings = LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: None,
            ..Default::default()
        };
        self.layout.reset(&settings);

        self.layout.append(
            &[&self.font],
            &TextStyle::new(text, size * self.device_pixel_ratio, 0),
        );

        let mut last_end_offset = Vector2::zeros();

        for glyph_pos in self.layout.glyphs() {
            // Ignore empty glyphs (e.g. space).
            if glyph_pos.width == 0 || glyph_pos.height == 0 {
                continue;
            }

            let (font, packer, bitmap_buffer) =
                (&self.font, &mut self.packer, &mut self.bitmap_buffer);

            let mut scaled_glyph_pos = glyph_pos.clone();
            let device_pixel_ratio = self.device_pixel_ratio;
            scaled_glyph_pos.key.px *= device_pixel_ratio;

            let glyph = self.cache.entry(scaled_glyph_pos.key).or_insert_with(|| {
                let (metrics, alpha_bitmap) = font.rasterize_indexed(
                    glyph_pos.key.glyph_index as usize,
                    size * device_pixel_ratio,
                );

                Self::alpha_to_rgba(&alpha_bitmap, bitmap_buffer);

                let uv_rect = packer
                    .insert(bitmap_buffer.as_slice(), metrics.width, metrics.height)
                    .unwrap(); // TODO: unwrap in atlas insert

                Glyph { uv_rect }
            });

            let rect_center = Point2::new(
                pos.x + (glyph_pos.x + glyph_pos.width as f32 / 2.0) / self.device_pixel_ratio,
                pos.y + (glyph_pos.y + glyph_pos.height as f32 / 2.0) / self.device_pixel_ratio,
            );
            let rect_size = Vector2::new(glyph_pos.width as f32, glyph_pos.height as f32)
                / self.device_pixel_ratio;

            batch.push_quad(
                &Quad::axis_aligned(rect_center, rect_size),
                pos.z,
                glyph.uv_rect,
                color,
            );

            last_end_offset = Vector2::new(
                glyph_pos.x + glyph_pos.width as f32 / 2.0,
                glyph_pos.y + glyph_pos.height as f32 / 2.0,
            ) / self.device_pixel_ratio;
        }

        last_end_offset
    }

    pub fn draw(
        &mut self,
        ctx: &Canvas,
        transform: &Matrix3<f32>,
        draw_unit: &DrawUnit<TexColVertex>,
    ) -> Result<(), Error> {
        self.device_pixel_ratio = ctx.screen().device_pixel_ratio as f32;

        ctx.golem_ctx().set_blend_mode(Some(BlendMode::default()));

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
            output.extend_from_slice(&[255, 255, 255, v]);
        }
    }
}
