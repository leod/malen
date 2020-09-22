use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use golem::{glow, GolemError};
use nalgebra as na;

use crate::{Error, Input, Matrix3, Vector2};

#[derive(Debug, Clone)]
pub struct Screen {
    /// The screen size in pixels.
    pub size: Vector2,
}

impl Screen {
    /// Returns an orthographic projection matrix.
    ///
    /// The returned matrix maps `[0..width] x [0..height]` to
    /// `[-1..1] x [-1..1]` (i.e. the OpenGL normalized device coordinates).
    /// Z coordinates are not transformed at all.
    ///
    /// Note that this projection also flips the Y axis, so that (0,0) is at
    /// the top-left of your screen.
    pub fn orthographic_projection(&self) -> Matrix3 {
        let scale_to_unit = na::Matrix3::new_nonuniform_scaling(&Vector2::new(
            1.0 / self.size.x,
            1.0 / self.size.y,
        ));
        let shift = na::Matrix3::new_translation(&-Vector2::new(-0.5, -0.5));
        let scale_and_flip_y = na::Matrix3::new_nonuniform_scaling(&Vector2::new(2.0, -2.0));

        scale_and_flip_y * shift * scale_to_unit
    }
}

pub struct Context {
    canvas: HtmlCanvasElement,
    input: Input,
    golem_context: golem::Context,
}

impl Context {
    pub fn from_canvas_id(id: &str) -> Result<Self, Error> {
        let canvas = web_sys::window()
            .ok_or(Error::NoWindow)?
            .document()
            .ok_or(Error::NoDocument)?
            .get_element_by_id(id)
            .ok_or_else(|| Error::InvalidElementId(id.into()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| Error::ElementIsNotCanvas(id.into()))?;

        Self::from_canvas_element(canvas)
    }

    pub fn from_canvas_element(canvas: HtmlCanvasElement) -> Result<Self, Error> {
        let input = Input::new(&canvas)?;
        let webgl_context = canvas
            .get_context("webgl")
            .map_err(Error::GetContext)?
            .ok_or(Error::InitializeWebGl)?
            .dyn_into::<WebGlRenderingContext>()
            .map_err(|_| Error::InitializeWebGl)?;
        let glow_context = glow::Context::from_webgl1_context(webgl_context);
        let golem_context = golem::Context::from_glow(glow_context)?;

        Ok(Context {
            canvas,
            input,
            golem_context,
        })
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn screen(&self) -> Screen {
        Screen {
            size: Vector2::new(self.canvas.width() as f32, self.canvas.height() as f32),
        }
    }

    pub fn golem_context(&self) -> &golem::Context {
        &self.golem_context
    }
}

impl From<GolemError> for Error {
    fn from(e: GolemError) -> Self {
        Error::Golem(e)
    }
}
