//! `malen` is yet another one of these libraries for 2D web game
//! development.

mod canvas;
mod color;
mod error;
mod input;

pub(crate) mod util;

pub mod draw;
pub mod geom;

// Re-export dependencies that occur in our public API.
pub use golem;
pub use golem::glow;
pub use nalgebra;

pub use golem::Texture;

pub use canvas::Canvas;
pub use color::{Color3, Color4};
pub use error::Error;
pub use geom::{AaRect, Camera, ScreenGeom};
pub use input::{Event, InputState, Key};
