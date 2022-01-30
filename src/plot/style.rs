use nalgebra::Vector2;

use crate::Color4;

#[derive(Debug, Clone)]
pub struct PlotStyle {
    pub axis_margin: Vector2<f32>,
    pub tick_size: f32,
    pub legend_line_size: f32,
    pub legend_text_margin: f32,
    pub legend_entry_margin: f32,
    pub legend_y_offset: f32,
    pub text_size: f32,
    pub tic_precision: usize,
    pub background_color: Option<Color4>,
    pub axis_color: Color4,
    pub text_color: Color4,
}

impl Default for PlotStyle {
    fn default() -> Self {
        Self {
            axis_margin: Vector2::new(60.0, 30.0),
            tick_size: 7.5,
            legend_line_size: 36.0,
            legend_text_margin: 9.0,
            legend_entry_margin: 36.0,
            legend_y_offset: 15.0,
            text_size: 17.0,
            tic_precision: 1,
            background_color: Some(Color4::new(0.4, 0.4, 0.7, 0.8)),
            axis_color: Color4::new(0.0, 0.0, 0.0, 1.0),
            text_color: Color4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

impl PlotStyle {
    pub fn graph_size(&self, plot_size: Vector2<f32>) -> Vector2<f32> {
        plot_size - 2.0 * self.axis_margin
    }
}
