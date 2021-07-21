use nalgebra::{convert, Matrix2, Matrix3, Point2, Point3, Vector2};

use crate::{
    draw::{ColPass, ColVertex, Font, LineBatch, TextBatch, TriBatch},
    AaRect, Canvas, Color4, Error,
};

const AXIS_MARGIN: f64 = 70.0;
const TICK_SIZE: f64 = 7.5;

const LEGEND_LINE_SIZE: f32 = 30.0;
const LEGEND_TEXT_MARGIN: f32 = 7.5;
const LEGEND_ENTRY_MARGIN: f32 = 50.0;
const LEGEND_Y_OFFSET: f32 = 15.0;

const FONT_SIZE: f32 = 14.0;

#[derive(Debug, Clone)]
pub struct Line {
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
    pub lines: Vec<Line>,
}

impl Plot {
    pub fn x_range(&self) -> (f64, f64) {
        self.x_axis.get_range(
            self.lines
                .iter()
                .flat_map(|line| line.points.iter().map(|(x, _)| *x)),
        )
    }

    pub fn y_range(&self) -> (f64, f64) {
        self.y_axis.get_range(
            self.lines
                .iter()
                .flat_map(|line| line.points.iter().map(|(_, y)| *y)),
        )
    }
}

pub struct Plotting {
    tri_batch: TriBatch<ColVertex>,
    line_batch: LineBatch<ColVertex>,
    text_batch: TextBatch,
    col_pass: ColPass,
}

struct RenderCtx<'a> {
    tri_batch: &'a mut TriBatch<ColVertex>,
    line_batch: &'a mut LineBatch<ColVertex>,
    text_batch: &'a mut TextBatch,
    font: &'a mut Font,
    plot: &'a Plot,
    x_range: (f64, f64),
    y_range: (f64, f64),
    plot_offset: Vector2<f64>,
    plot_size: Vector2<f64>,
    axis_scale: Matrix2<f64>,
}

impl<'a> RenderCtx<'a> {
    fn new(plotting: &'a mut Plotting, font: &'a mut Font, plot: &'a Plot) -> Self {
        let plot_offset = Vector2::new(AXIS_MARGIN / 2.0, AXIS_MARGIN / 2.0);
        let plot_size = plot.size - 2.0 * plot_offset;
        let x_range = plot.x_range();
        let y_range = plot.y_range();
        let axis_scale = Matrix2::from_diagonal(&Vector2::new(
            plot_size.x / (x_range.1 - x_range.0),
            plot_size.y / (y_range.1 - y_range.0),
        ));

        Self {
            tri_batch: &mut plotting.tri_batch,
            line_batch: &mut plotting.line_batch,
            text_batch: &mut plotting.text_batch,
            font,
            plot,
            x_range,
            y_range,
            plot_offset,
            plot_size,
            axis_scale,
        }
    }

    fn render_background(&mut self) {
        if let Some(background_color) = self.plot.background_color {
            self.tri_batch.push_quad(
                &AaRect::from_top_left(Point2::origin(), convert(self.plot.size)).into(),
                0.0,
                background_color,
            );
        }
    }

    fn render_outline(&mut self) {
        self.line_batch.push_quad_outline(
            &AaRect::from_top_left(
                convert(Point2::origin() + self.plot_offset),
                convert(self.plot_size),
            )
            .into(),
            0.1,
            self.plot.axis_color,
        );
    }

    fn map_point(&mut self, pos: Point2<f64>) -> Point2<f32> {
        let clipped = clip_to_ranges(pos, self.x_range, self.y_range);
        let shifted = clipped - Vector2::new(self.x_range.0, self.y_range.0);
        let scaled = self.axis_scale * shifted;
        let margined = scaled + self.plot_offset;
        let flipped = Point2::new(margined.x, self.plot.size.y - margined.y);
        convert(flipped)
    }

    fn render_tics(&mut self, range: (f64, f64), tics: f64, tic_precision: usize, is_x: bool) {
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

        let epsilon = 1e-7;
        let tics_rem = range.0.rem_euclid(tics);
        let mut current_offset = if tics_rem < epsilon {
            round_to_multiple(range.0)
        } else {
            range.0 + tics - tics_rem
        };

        while current_offset <= range.1 {
            let text = &format!("{:.*}", tic_precision, current_offset);
            let text_size = self.font.text_size(11.0, &text);

            let pos = self.map_point(
                Point2::new(self.x_range.0, self.y_range.0) + (current_offset - range.0) * axis,
            );
            let shifted_pos = if is_x {
                Point2::new(pos.x - text_size.x / 2.0, pos.y + 6.0)
            } else {
                Point2::new(pos.x - 8.0 - text_size.x, pos.y - text_size.y / 2.0 - 3.0)
            };
            let shifted_pos = Point3::new(shifted_pos.x, shifted_pos.y, 0.2);

            self.font.write(
                FONT_SIZE,
                convert(shifted_pos),
                self.plot.text_color,
                &text,
                &mut self.text_batch,
            );

            if is_x {
                self.line_batch.push_line(
                    pos,
                    pos + normal * TICK_SIZE as f32,
                    0.4,
                    self.plot.axis_color,
                );
                self.line_batch.push_line(
                    pos - Vector2::new(0.0, self.plot_size.y as f32),
                    pos - Vector2::new(0.0, self.plot_size.y as f32) - normal * TICK_SIZE as f32,
                    0.4,
                    self.plot.axis_color,
                );
            } else {
                self.line_batch.push_line(
                    pos,
                    pos + normal * TICK_SIZE as f32,
                    0.4,
                    self.plot.axis_color,
                );
                self.line_batch.push_line(
                    pos + Vector2::new(self.plot_size.x as f32, 0.0),
                    pos + Vector2::new(self.plot_size.x as f32, 0.0) - normal * TICK_SIZE as f32,
                    0.4,
                    self.plot.axis_color,
                );
            }

            current_offset = round_to_multiple(current_offset + tics);
        }
    }

    fn render_legend(&mut self) {
        if self.plot.lines.is_empty() {
            return;
        }

        let mut width = (LEGEND_LINE_SIZE + LEGEND_TEXT_MARGIN) * self.plot.lines.len() as f32
            + LEGEND_ENTRY_MARGIN as f32 * ((self.plot.lines.len() - 1) as f32);
        let mut max_text_height = 0.0f32;
        for line in self.plot.lines.iter() {
            let size = self.font.text_size(FONT_SIZE, &line.caption);
            width += size.x;
            max_text_height = max_text_height.max(size.y / 2.0);
        }

        let mut pos = Point2::new(self.plot.size.x as f32 / 2.0 - width / 2.0, LEGEND_Y_OFFSET);
        for line in self.plot.lines.iter() {
            let line_start = pos;
            pos.x += LEGEND_LINE_SIZE;
            self.line_batch.push_line(line_start, pos, 0.3, line.color);

            pos.x += LEGEND_TEXT_MARGIN;
            let text_size = self.font.write(
                FONT_SIZE,
                Point3::new(pos.x, pos.y - max_text_height - 1.0, 0.3),
                self.plot.text_color,
                &line.caption,
                self.text_batch,
            );
            pos.x += text_size.x;

            pos.x += LEGEND_ENTRY_MARGIN;
        }
    }

    fn render_lines(&mut self) {
        for line in self.plot.lines.iter() {
            for (p, q) in line.points.iter().zip(line.points.iter().skip(1)) {
                let p = self.map_point(Point2::new(p.0, p.1));
                let q = self.map_point(Point2::new(q.0, q.1));
                self.line_batch.push_line(p, q, 0.3, line.color);
            }
        }
    }

    fn render(&mut self) {
        self.render_background();
        self.render_lines();
        self.render_outline();
        self.render_tics(
            self.x_range,
            self.plot.x_axis.tics,
            self.plot.x_axis.tic_precision,
            true,
        );
        self.render_tics(
            self.y_range,
            self.plot.y_axis.tics,
            self.plot.y_axis.tic_precision,
            false,
        );
        self.render_legend();
    }
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

        // Collect things to be drawn into the batches.
        let mut ctx = RenderCtx::new(self, font, plot);
        ctx.render();

        // Finally, draw all the prepared batches.
        self.col_pass
            .draw(&transform, &self.tri_batch.draw_unit())?;
        self.col_pass
            .draw(&transform, &self.line_batch.draw_unit())?;
        font.draw(canvas, &transform, &self.text_batch.draw_unit())?;

        Ok(())
    }
}
