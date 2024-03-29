use nalgebra::{Matrix3, Point2, Vector2};

use super::Rect;

#[derive(Debug, Clone, Copy)]
pub struct Screen {
    pub logical_size: Vector2<f32>,
    pub physical_size: Vector2<u32>,
    pub device_pixel_ratio: f64,
}

impl Screen {
    /// Returns an orthographic projection matrix.
    ///
    /// The returned matrix maps `[0..width] x [0..height]` to
    /// `[-1..1] x [-1..1]` (i.e. the OpenGL normalized device coordinates).
    ///
    /// Notes:
    /// - This projection also flips the Y axis, so that (0,0) is at the
    ///   top-left of your screen.
    /// - We assume the Z coordinate of the input vector to be set to 1.
    pub fn project_to_ndc(input_size: Vector2<f32>) -> Matrix3<f32> {
        let scale_to_unit =
            Matrix3::new_nonuniform_scaling(&Vector2::new(1.0 / input_size.x, 1.0 / input_size.y));
        let shift = Matrix3::new_translation(&Vector2::new(-0.5, -0.5));
        let scale_and_flip_y = Matrix3::new_nonuniform_scaling(&Vector2::new(2.0, -2.0));

        scale_and_flip_y * shift * scale_to_unit
    }

    pub fn project_logical_to_ndc(&self) -> Matrix3<f32> {
        Self::project_to_ndc(self.logical_size)
    }

    pub fn project_physical_to_ndc(&self) -> Matrix3<f32> {
        Self::project_to_ndc(self.physical_size.cast())
    }

    pub fn physical_rect(&self) -> Rect {
        Rect::from_top_left(Point2::origin(), nalgebra::convert(self.physical_size))
    }

    pub fn logical_rect(&self) -> Rect {
        Rect::from_top_left(Point2::origin(), nalgebra::convert(self.logical_size))
    }
}
