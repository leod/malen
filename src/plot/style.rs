use nalgebra::Vector2;

use crate::Color4;

#[derive(Debug, Clone)]
pub struct PlotStyle {
    pub axis_margin: f32,
    pub tick_size: f32,
    pub legend_line_size: f32,
    pub legend_text_margin: f32,
    pub legend_entry_margin: f32,
    pub legend_y_offset: f32,
    pub normal_font_size: f32,
    pub small_font_size: f32,
    pub tic_precision: usize,
    pub background_color: Option<Color4>,
    pub axis_color: Color4,
    pub text_color: Color4,
}

impl Default for PlotStyle {
    fn default() -> Self {
        Self {
            axis_margin: 70.0,
            tick_size: 7.5,
            legend_line_size: 30.0,
            legend_text_margin: 7.5,
            legend_entry_margin: 50.0,
            legend_y_offset: 15.0,
            normal_font_size: 14.0,
            small_font_size: 11.0,
            tic_precision: 1,
            background_color: Some(Color4::new(0.2, 0.2, 0.4, 0.5)),
            axis_color: Color4::new(0.0, 0.0, 0.0, 1.0),
            text_color: Color4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

impl PlotStyle {
    pub fn graph_offset(&self) -> Vector2<f32> {
        Vector2::new(self.axis_margin, self.axis_margin)
    }

    pub fn graph_size(&self, plot_size: Vector2<f32>) -> Vector2<f32> {
        plot_size - 2.0 * self.graph_offset()
    }
}
