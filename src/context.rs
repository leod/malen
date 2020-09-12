use thiserror::Error;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use golem::{glow, GolemError};

use crate::Input;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no window")]
    NoWindow,

    #[error("no document")]
    NoDocument,

    #[error("could not find HTML element with id `{0}`")]
    InvalidElementId(String),

    #[error("HTML element with id `{0}` is not a canvas")]
    ElementIsNotCanvas(String),

    #[error("error from golem crate: {0}")]
    Golem(GolemError),

    #[error("could not get WebGL1 context")]
    GetContext(JsValue),

    #[error("could not initialize WebGL1")]
    InitializeWebGl,
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
}

impl From<GolemError> for Error {
    fn from(e: GolemError) -> Self {
        Error::Golem(e)
    }
}
