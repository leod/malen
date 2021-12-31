use nalgebra::{Matrix3, Point2, Vector2};

use super::{translate_rotate_scale, Screen};

#[derive(Debug, Clone)]
/// Parameters that define a two-dimensional camera transformation.
pub struct Camera {
    /// The center position of the camera.
    pub center: Point2<f32>,

    /// Zoom factor. For values larger than one, things will look larger, while
    /// for values smaller than one they will look smaller.
    /// Using nonpositive values will probably lead to funny (or frustrating)
    /// results.
    pub zoom: f32,

    /// Angle of rotation around the center in radians.
    pub angle: f32,
}

impl Camera {
    pub fn screen_view_matrix(screen: &Screen) -> Matrix3<f32> {
        Self {
            center: Point2::new(
                screen.logical_size.x as f32 / 2.0,
                screen.logical_size.y as f32 / 2.0,
            ),
            angle: 0.0,
            zoom: 1.0,
        }
        .to_matrix(screen)
    }

    /// Build a 3x3 matrix with homogeneous coordinates to represent the
    /// transformation from world space to camera space.
    pub fn to_matrix(&self, screen: &Screen) -> Matrix3<f32> {
        // It's a bit easier to first consider the camera space -> world space
        // transformation C2W and then take the inverse to get W2C. For C2W, we
        // first need to scale with S / rotate with R (order shouldn't matter
        // since we scale all axes the same), and then translate with T:
        //
        //     C2W = T(center) * S(1.0 / zoom) * R(angle)
        //
        //     => W2C = C2W^-1 = S(zoom) * R(-angle) * T(-center)
        //
        // (Using S(x)^-1 = S(1/x),
        //        R(x)^-1 = R(-x),
        //        T(x)^-1 = T(-x).)

        Matrix3::new_translation(&Vector2::new(
            screen.logical_size.x as f32 / 2.0,
            screen.logical_size.y as f32 / 2.0,
        )) * translate_rotate_scale(
            -self.center.coords,
            -self.angle,
            Vector2::new(self.zoom, self.zoom),
        )
    }
}
