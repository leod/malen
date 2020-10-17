use std::{collections::HashMap, ops::Deref};

use fontdue::{
    layout::{CoordinateSystem, GlyphPosition, Layout, LayoutSettings, TextStyle},
    FontSettings,
};
use golem::blend::{BlendEquation, BlendFactor, BlendFunction, BlendMode, BlendOperation};

use crate::{
    draw::{text::packer::ShelfPacker, Batch, Quad, TexColPass, TexColVertex},
    Color, Context, Error, Matrix3, Point3, Rect, Vector2,
};

struct Glyph {
    uv_rect: Rect,
}

pub struct Font {
    font: fontdue::Font,
    scale: f32,
    layout: Layout,

    packer: ShelfPacker,
    cache: HashMap<char, Glyph>,

    pass: TexColPass,

    position_buffer: Vec<GlyphPosition>,
    bitmap_buffer: Vec<u8>,
}

const ATLAS_WIDTH: usize = 2048;
const ATLAS_HEIGHT: usize = 2048;

impl Font {
    pub fn from_bytes<Data>(ctx: &Context, data: Data, scale: f32) -> Result<Self, Error>
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
            scale,
            packer,
            layout,
            cache: HashMap::new(),
            pass,
            position_buffer: Vec::new(),
            bitmap_buffer: Vec::new(),
        })
    }

    pub fn write(
        &mut self,
        batch: &mut Batch<TexColVertex>,
        pos: Point3,
        color: Color,
        text: &str,
    ) {
        self.position_buffer.clear();

        let settings = LayoutSettings {
            x: pos.x,
            y: pos.y,
            max_width: None,
            ..Default::default()
        };

        self.layout.layout_horizontal(
            &[&self.font],
            &[&TextStyle::new(text, self.scale, 0)],
            &settings,
            &mut self.position_buffer,
        );

        for &glyph_pos in &self.position_buffer {
            let (scale, font, packer, bitmap_buffer) = (
                self.scale,
                &self.font,
                &mut self.packer,
                &mut self.bitmap_buffer,
            );

            let glyph = self.cache.entry(glyph_pos.key.c).or_insert_with(|| {
                let (metrics, alpha_bitmap) = font.rasterize(glyph_pos.key.c, scale);

                Self::alpha_to_rgba(&alpha_bitmap, bitmap_buffer);

                let uv_rect = packer
                    .insert(bitmap_buffer.as_slice(), metrics.width, metrics.height)
                    .unwrap(); // TODO: unwrap in atlas insert

                Glyph { uv_rect }
            });

            let rect_center = Point3::new(
                glyph_pos.x + glyph_pos.width as f32 / 2.0,
                glyph_pos.y + glyph_pos.height as f32 / 2.0,
                pos.z,
            );
            let rect_size = Vector2::new(glyph_pos.width as f32, glyph_pos.height as f32);

            batch.push_quad(
                &Quad::axis_aligned(rect_center, rect_size),
                glyph.uv_rect,
                color,
            );
        }
    }

    pub fn draw_batch(
        &mut self,
        ctx: &Context,
        projection: &Matrix3,
        batch: &mut Batch<TexColVertex>,
    ) -> Result<(), Error> {
        batch.flush();

        ctx.golem_ctx().set_blend_mode(Some(BlendMode {
            equation: BlendEquation::Same(BlendOperation::Add),
            function: BlendFunction::Same {
                source: BlendFactor::One,
                destination: BlendFactor::One,
            },
            ..Default::default()
        }));

        self.pass.draw_batch(
            projection,
            &Matrix3::identity(),
            self.packer.texture(),
            batch,
        )?;

        ctx.golem_ctx().set_blend_mode(None);

        Ok(())
    }

    fn alpha_to_rgba(bitmap: &[u8], output: &mut Vec<u8>) {
        output.clear();
        for v in bitmap {
            let v = *v;
            output.extend_from_slice(&[v, v, v, v]);
        }
    }
}
