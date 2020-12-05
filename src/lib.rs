//! `webglee` is yet another one of these libraries for 2D web game
//! development.

mod color;
mod context;
mod input;

pub mod draw;
pub mod geom;

// Re-export dependencies that occur in our public API.
pub use golem;
pub use golem::glow;
pub use nalgebra;

pub use golem::Texture;

pub use color::{Color3, Color4};
pub use context::Context;
pub use draw::Draw;
pub use geom::{AaRect, Camera, Screen};
pub use input::{Event, InputState, Key};

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
    GetContext(String),

    #[error("could not initialize WebGL1")]
    InitializeWebGl,

    #[error("Failed to load font")]
    Font(String),
}
