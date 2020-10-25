//! `webglee` is yet another one of these libraries for 2D web game
//! development.

mod context;
mod input;

pub mod draw;
pub mod geom;
pub mod prelude;

// Re-export dependencies that occur in our public API.
pub use golem;
pub use golem::glow;
pub use nalgebra;

pub use context::Context;
pub use draw::Draw;
pub use geom::{Camera, Rect, Screen};
pub use input::{Event, InputState, VirtualKeyCode};
pub use prelude::{Color, Matrix2, Matrix3, Point2, Point3, Vector2, Vector3};

use golem::GolemError;
use thiserror::Error;
use wasm_bindgen::JsValue;

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

    #[error("Failed to load font")]
    Font(String),
}
