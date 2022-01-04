//! `malen` is yet another one of these libraries for 2D web game
//! development.

mod canvas;
mod color;
mod config;
mod context;
mod error;
mod input;
mod main_loop;
mod pass;
mod plot;

pub(crate) mod util;

pub mod geometry;
pub mod gl;
pub mod math;
pub mod text;

// Re-export dependencies that occur in our public API.
pub use glow;
pub use nalgebra;

pub use canvas::{Canvas, CanvasSizeConfig};
pub use color::{Color3, Color4};
pub use config::Config;
pub use context::Context;
pub use error::{FrameError, InitError};
pub use input::{Event, InputState, Key};
pub use main_loop::main_loop;
pub use math::{Camera, Rect, RotatedRect, Screen};
pub use pass::MatricesBlock;
