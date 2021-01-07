use nalgebra::{Matrix2, Matrix3, Point2, Vector2};

use crate::{
    draw::{ColPass, ColVertex, Font, LineBatch, TextBatch, TriBatch},
    AaRect, Canvas, Color4, Error,
};

const AXIS_MARGIN: f32 = 30.0;

#[derive(Debug, Clone)]
pub struct LinePlotData {
    pub caption: String,
    pub color: Color4,
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone)]
pub struct Axis {
    pub label: String,
    pub range: Option<(f32, f32)>,
    pub tics: f32,
}

impl Axis {
    pub fn get_range(&self, data: impl Iterator<Item = f32>) -> (f32, f32) {
        self.range.unwrap_or_else(|| {
            let mut min = std::f32::MAX;
            let mut max = std::f32::MIN;

            for item in data {
                min = min.min(item);
                max = max.max(item);
            }

            (min, max)
        })
    }
}

fn clip_to_ranges(p: Point2<f32>, x_range: (f32, f32), y_range: (f32, f32)) -> Point2<f32> {
    Point2::new(
        p.x.max(x_range.0).min(x_range.1),
        p.y.max(y_range.0).min(y_range.1),
    )
}

#[derive(Debug, Clone)]
pub struct Plot {
    pub size: Vector2<f32>,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub axis_color: Color4,
    pub background_color: Option<Color4>,
    pub lines: Vec<LinePlotData>,
}

pub struct Plotting {
    tri_batch: TriBatch<ColVertex>,
    line_batch: LineBatch<ColVertex>,
    text_batch: TextBatch,
    col_pass: ColPass,
}

impl Plotting {
    pub fn new(canvas: &Canvas) -> Result<Self, Error> {
        Ok(Self {
            tri_batch: TriBatch::new(canvas)?,
            line_batch: LineBatch::new(canvas)?,
            text_batch: TextBatch::new(canvas)?,
            col_pass: ColPass::new(canvas)?,
        })
    }

    pub fn draw(
        &mut self,
        canvas: &Canvas,
        font: &mut Font,
        transform: &Matrix3<f32>,
        plot: &Plot,
    ) -> Result<(), Error> {
        // Flip Y axis so that (0,0) is at the bottom-right of the screen again.
        // (This is assuming that the caller uses
        // ScreenGeom::orthographic_projection.)
        let transform = Matrix3::new_nonuniform_scaling(&Vector2::new(1.0, -1.0)) * transform;

        // Reset batch contents from previous draw calls.
        self.tri_batch.clear();
        self.line_batch.clear();
        self.text_batch.clear();

        // Render plot background.
        if let Some(background_color) = plot.background_color {
            self.tri_batch.push_quad(
                &AaRect::from_top_left(Point2::origin(), plot.size).into(),
                0.0,
                background_color,
            );
        }

        // Render X and Y axis.
        let plot_offset = Vector2::new(AXIS_MARGIN / 2.0, AXIS_MARGIN / 2.0);
        let plot_size = plot.size - 2.0 * plot_offset;

        self.line_batch.push_quad_outline(
            &AaRect::from_top_left(Point2::origin() + plot_offset, plot_size).into(),
            0.1,
            plot.axis_color,
        );

        // Determine min/max X and Y values, or use the user's defined ranges.
        let x_range = plot.x_axis.get_range(
            plot.lines
                .iter()
                .flat_map(|line| line.points.iter().map(|(x, _)| *x)),
        );
        let y_range = plot.y_axis.get_range(
            plot.lines
                .iter()
                .flat_map(|line| line.points.iter().map(|(_, y)| *y)),
        );

        let axis_scale = Matrix2::from_diagonal(&Vector2::new(
            plot_size.x / (x_range.1 - x_range.0),
            plot_size.y / (y_range.1 - y_range.0),
        ));

        let map_point = |(x, y): (f32, f32)| {
            let pos = Point2::new(x, y);
            let clipped = clip_to_ranges(pos, x_range, y_range);
            let shifted = clipped - Vector2::new(x_range.0, y_range.0);
            let scaled = axis_scale * shifted;
            let margined = scaled + plot_offset;
            margined
        };

        // Render each of the lines.
        for line in plot.lines.iter() {
            for (p, q) in line.points.iter().zip(line.points.iter().skip(1)) {
                self.line_batch
                    .push_line(map_point(*p), map_point(*q), 0.2, line.color);
            }
        }

        // Finally, draw all the prepared batches.
        self.col_pass
            .draw(&transform, &self.tri_batch.draw_unit())?;
        self.col_pass
            .draw(&transform, &self.line_batch.draw_unit())?;
        font.draw(canvas, &transform, &self.text_batch.draw_unit())?;

        Ok(())
    }

    fn render_tics(
        &mut self,
        axis: &Axis,
        start_pos: Point2<f32>,
        delta: Vector2<f32>,
        offset: Vector2<f32>,
        n: usize,
        font: &Font,
    ) {
    }
}
