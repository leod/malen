use nalgebra::{convert, Matrix2, Matrix3, Point2, Point3, Vector2};

use crate::{
    draw::{ColPass, ColVertex, Font, LineBatch, TextBatch, TriBatch},
    AaRect, Canvas, Color4, Error,
};

const AXIS_MARGIN: f64 = 70.0;
const TICK_SIZE: f64 = 7.5;

#[derive(Debug, Clone)]
pub struct LinePlotData {
    pub caption: String,
    pub color: Color4,
    pub points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct Axis {
    pub label: String,
    pub range: Option<(f64, f64)>,
    pub tics: f64,
    pub tic_precision: usize,
}

impl Axis {
    pub fn get_range(&self, data: impl Iterator<Item = f64>) -> (f64, f64) {
        self.range.unwrap_or_else(|| {
            let mut min = std::f64::MAX;
            let mut max = std::f64::MIN;

            for item in data {
                min = min.min(item);
                max = max.max(item);
            }

            (min, max)
        })
    }
}

fn clip_to_ranges(p: Point2<f64>, x_range: (f64, f64), y_range: (f64, f64)) -> Point2<f64> {
    Point2::new(
        p.x.max(x_range.0).min(x_range.1),
        p.y.max(y_range.0).min(y_range.1),
    )
}

#[derive(Debug, Clone)]
pub struct Plot {
    pub size: Vector2<f64>,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub axis_color: Color4,
    pub background_color: Option<Color4>,
    pub text_color: Color4,
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
        // Reset batch contents from previous draw calls.
        self.tri_batch.clear();
        self.line_batch.clear();
        self.text_batch.clear();

        // Render plot background.
        if let Some(background_color) = plot.background_color {
            self.tri_batch.push_quad(
                &AaRect::from_top_left(Point2::origin(), convert(plot.size)).into(),
                0.0,
                background_color,
            );
        }

        // Render X and Y axis.
        let plot_offset = Vector2::new(AXIS_MARGIN / 2.0, AXIS_MARGIN / 2.0);
        let plot_size = plot.size - 2.0 * plot_offset;

        self.line_batch.push_quad_outline(
            &AaRect::from_top_left(convert(Point2::origin() + plot_offset), convert(plot_size))
                .into(),
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

        let map_point = |pos: Point2<f64>| -> Point2<f32> {
            let clipped = clip_to_ranges(pos, x_range, y_range);
            let shifted = clipped - Vector2::new(x_range.0, y_range.0);
            let scaled = axis_scale * shifted;
            let margined = scaled + plot_offset;
            let flipped = Point2::new(margined.x, plot.size.y - margined.y);
            convert(flipped)
        };

        // Draw tics at the axes.
        let mut render_tics = |range: (f64, f64), tics: f64, tic_precision: usize, is_x: bool| {
            if range.0 >= range.1 || tics == 0.0 {
                return;
            }

            let round_to_multiple = |x: f64| (x / tics).round() * tics;

            let axis = if is_x {
                Vector2::new(1.0, 0.0)
            } else {
                Vector2::new(0.0, 1.0)
            };
            let normal = if is_x {
                Vector2::new(0.0f32, -1.0)
            } else {
                Vector2::new(1.0, 0.0f32)
            };

            let mut current_offset = round_to_multiple(range.0);
            while current_offset <= round_to_multiple(range.1) {
                let text = &format!("{:.*}", tic_precision, current_offset);
                let text_size = font.text_size(11.0, &text);

                let pos = map_point(
                    Point2::new(x_range.0, y_range.0) + (current_offset - range.0) * axis,
                );
                let shifted_pos = if is_x {
                    Point2::new(pos.x - text_size.x / 2.0, pos.y + 6.0)
                } else {
                    Point2::new(pos.x - 6.0 - text_size.x, pos.y - text_size.y / 2.0 - 3.0)
                };
                let shifted_pos = Point3::new(shifted_pos.x, shifted_pos.y, 0.2);

                font.write(
                    11.0,
                    convert(shifted_pos),
                    plot.text_color,
                    &text,
                    &mut self.text_batch,
                );

                if is_x {
                    self.line_batch.push_line(
                        pos,
                        pos + normal * TICK_SIZE as f32,
                        0.4,
                        plot.axis_color,
                    );
                    self.line_batch.push_line(
                        pos - Vector2::new(0.0, plot_size.y as f32),
                        pos - Vector2::new(0.0, plot_size.y as f32) - normal * TICK_SIZE as f32,
                        0.4,
                        plot.axis_color,
                    );
                } else {
                    self.line_batch.push_line(
                        pos,
                        pos + normal * TICK_SIZE as f32,
                        0.4,
                        plot.axis_color,
                    );
                    self.line_batch.push_line(
                        pos + Vector2::new(plot_size.x as f32, 0.0),
                        pos + Vector2::new(plot_size.x as f32, 0.0) - normal * TICK_SIZE as f32,
                        0.4,
                        plot.axis_color,
                    );
                }

                current_offset = round_to_multiple(current_offset + tics);
            }
        };

        render_tics(x_range, plot.x_axis.tics, plot.x_axis.tic_precision, true);
        render_tics(y_range, plot.y_axis.tics, plot.y_axis.tic_precision, false);

        // Render each of the lines.
        for line in plot.lines.iter() {
            for (p, q) in line.points.iter().zip(line.points.iter().skip(1)) {
                self.line_batch.push_line(
                    map_point(Point2::new(p.0, p.1)),
                    map_point(Point2::new(q.0, q.1)),
                    0.3,
                    line.color,
                );
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
}
