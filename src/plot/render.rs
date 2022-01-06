use std::rc::Rc;

use nalgebra::{Point2, Vector2};

use crate::{
    data::{ColorLine, ColorLineBatch, ColorRect, ColorTriangleBatch},
    gl,
    text::{Font, Text, TextBatch, WriteTextError},
    Rect, math::Line,
};

use super::{Plot, PlotStyle};

pub struct PlotBatch {
    pub triangle_batch: ColorTriangleBatch,
    pub line_batch: ColorLineBatch,
    pub text_batch: TextBatch,
}

impl PlotBatch {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let triangle_batch = ColorTriangleBatch::new(gl.clone())?;
        let line_batch = ColorLineBatch::new(gl.clone())?;
        let text_batch = TextBatch::new(gl)?;

        Ok(Self {
            triangle_batch,
            line_batch,
            text_batch,
        })
    }

    pub fn clear(&mut self) {
        self.triangle_batch.clear();
        self.line_batch.clear();
        self.text_batch.clear();
    }

    pub fn push(
        &mut self,
        font: &mut Font,
        plot: Plot,
        style: PlotStyle,
    ) -> Result<(), WriteTextError> {
        Render::new(self, font, &plot, &style).render()
    }
}

struct Render<'a> {
    batch: &'a mut PlotBatch,

    font: &'a mut Font,
    plot: &'a Plot,
    style: &'a PlotStyle,

    x_range: (f32, f32),
    y_range: (f32, f32),
    graph_size: Vector2<f32>,
    axis_scale: Vector2<f32>,
}

impl<'a> Render<'a> {
    fn new(
        batch: &'a mut PlotBatch,
        font: &'a mut Font,
        plot: &'a Plot,
        style: &'a PlotStyle,
    ) -> Self {
        let x_range = plot.x_range();
        let y_range = plot.y_range();
        let graph_size = style.graph_size(plot.rect.size);
        let axis_scale = Vector2::new(
            graph_size.x / (x_range.1 - x_range.0) as f32,
            graph_size.y / (y_range.1 - y_range.0),
        );

        Self {
            batch,
            plot,
            style,
            font,
            x_range,
            y_range,
            graph_size,
            axis_scale,
        }
    }

    fn render(&mut self) -> Result<(), WriteTextError> {
        self.render_background();
        self.render_lines();
        self.render_border();
        self.render_tics(self.x_range, self.plot.x_axis.tics, true)?;
        self.render_tics(self.y_range, self.plot.y_axis.tics, false)?;
        self.render_legend()?;

        Ok(())
    }

    fn render_background(&mut self) {
        if let Some(background_color) = self.style.background_color {
            self.batch.triangle_batch.push(ColorRect {
                rect: self.plot.rect,
                z: 0.0,
                color: background_color,
            });
        }
    }

    fn render_lines(&mut self) {
        for line_graph in self.plot.line_graphs.iter() {
            for (a, b) in line_graph
                .points
                .iter()
                .zip(line_graph.points.iter().skip(1))
            {
                let a_map = self.map_point(Point2::new(a.0, a.1));
                let b_map = self.map_point(Point2::new(b.0, b.1));

                self.batch.line_batch.push(ColorLine {
                    line: Line(a_map, b_map),
                    z: 0.0,
                    color: line_graph.color,
                });
            }
        }
    }

    fn render_border(&mut self) {
        self.batch.line_batch.push(ColorRect {
            rect: Rect::from_top_left(
                self.plot.rect.top_left() + self.style.axis_margin,
                self.graph_size,
            ),
            z: 0.0,
            color: self.style.axis_color,
        });
    }

    fn render_tics(
        &mut self,
        range: (f32, f32),
        tics: f32,
        is_x: bool,
    ) -> Result<(), WriteTextError> {
        if range.0 >= range.1 || tics == 0.0 {
            return Ok(());
        }

        let round_to_tics = |x: f32| (x / tics).round() * tics;

        let (axis, normal) = if is_x {
            (Vector2::new(1.0, 0.0), Vector2::new(0.0, -1.0))
        } else {
            (Vector2::new(0.0, 1.0), Vector2::new(1.0, 0.0))
        };

        let epsilon = 1e-7;
        let tics_rem = range.0.rem_euclid(tics);
        let mut current_offset = if tics_rem < epsilon {
            round_to_tics(range.0)
        } else {
            range.0 + tics - tics_rem
        };

        while current_offset <= range.1 {
            let text = &format!("{:.*}", self.style.tic_precision, current_offset);
            let text_size = self.font.text_size(self.style.normal_font_size, &text);

            let pos = self.map_point(
                Point2::new(self.x_range.0, self.y_range.0) + (current_offset - range.0) * axis,
            );
            let shifted_pos = if is_x {
                pos - Vector2::new(text_size.x / 2.0, 0.0)
            } else {
                pos - Vector2::new(8.0 + text_size.x, text_size.y / 2.0 + 3.0)
            };

            self.font.write(
                Text {
                    pos: shifted_pos,
                    size: self.style.normal_font_size,
                    z: 0.0,
                    color: self.style.text_color,
                    text: &text,
                },
                &mut self.batch.text_batch,
            )?;

            if is_x {
                self.batch.line_batch.push(ColorLine {
                    line: Line(pos, pos + normal * self.style.tick_size),
                    z: 0.0,
                    color: self.style.axis_color,
                });
                self.batch.line_batch.push(ColorLine {
                    line: Line(
                        pos - Vector2::new(0.0, self.graph_size.y),
                        pos - Vector2::new(0.0, self.graph_size.y) - normal * self.style.tick_size,
                    ),
                    z: 0.0,
                    color: self.style.axis_color,
                });
            } else {
                self.batch.line_batch.push(ColorLine {
                    line: Line(pos, pos + normal * self.style.tick_size),
                    z: 0.0,
                    color: self.style.axis_color,
                });
                self.batch.line_batch.push(ColorLine {
                    line: Line(pos + Vector2::new(self.graph_size.x, 0.0), pos + Vector2::new(self.graph_size.x, 0.0) - normal * self.style.tick_size),
                    z: 0.0,
                    color: self.style.axis_color,
                });
            }

            current_offset = round_to_tics(current_offset + tics);
        }

        Ok(())
    }

    fn render_legend(&mut self) -> Result<(), WriteTextError> {
        if self.plot.line_graphs.is_empty() {
            return Ok(());
        }

        let mut width = 0.0;
        let mut max_text_height = 0.0f32;

        width += (self.style.legend_line_size + self.style.legend_text_margin)
            * self.plot.line_graphs.len() as f32;
        width += self.style.legend_entry_margin as f32 * ((self.plot.line_graphs.len() - 1) as f32);

        for line in self.plot.line_graphs.iter() {
            let text_size = self
                .font
                .text_size(self.style.normal_font_size, &line.caption);

            width += text_size.x;
            max_text_height = max_text_height.max(text_size.y);
        }

        let mut pos = self.plot.rect.top_left();

        pos += Vector2::new(
            self.plot.rect.size.x as f32 / 2.0 - width / 2.0,
            self.style.legend_y_offset,
        );

        for line in self.plot.line_graphs.iter() {
            self.batch.line_batch.push(ColorLine {
                line: Line(pos, pos + Vector2::new(self.style.legend_line_size, 0.0)),
                z: 0.0,
                color: line.color,
            });

            pos.x += self.style.legend_line_size + self.style.legend_text_margin;

            let text_size = self.font.write(
                Text {
                    pos: pos - Vector2::new(0.0, max_text_height / 2.0 + 2.0),
                    size: self.style.normal_font_size,
                    z: 0.0,
                    color: self.style.text_color,
                    text: &line.caption,
                },
                &mut self.batch.text_batch,
            )?;

            pos.x += text_size.x;
            pos.x += self.style.legend_entry_margin;
        }

        Ok(())
    }

    fn map_point(&mut self, pos: Point2<f32>) -> Point2<f32> {
        let shift = pos.coords - Vector2::new(self.x_range.0, self.y_range.0);
        let scale = shift.component_mul(&self.axis_scale);
        let margin = scale + self.style.axis_margin;
        let flip = Vector2::new(margin.x, self.plot.rect.size.y - margin.y);
        self.plot.rect.top_left() + flip
    }
}
