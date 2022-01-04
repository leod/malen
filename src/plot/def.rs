use crate::{Color4, Rect};

#[derive(Debug, Clone)]
pub struct LineGraph {
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

#[derive(Debug, Clone)]
pub struct Plot {
    pub rect: Rect,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub line_graphs: Vec<LineGraph>,
}

impl Plot {
    pub fn x_range(&self) -> (f32, f32) {
        self.x_axis.get_range(
            self.line_graphs
                .iter()
                .flat_map(|line| line.points.iter().map(|(x, _)| *x)),
        )
    }

    pub fn y_range(&self) -> (f32, f32) {
        self.y_axis.get_range(
            self.line_graphs
                .iter()
                .flat_map(|line| line.points.iter().map(|(_, y)| *y)),
        )
    }
}
