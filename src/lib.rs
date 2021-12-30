//! `malen` is yet another one of these libraries for 2D web game
//! development.

mod canvas;
mod color;
mod error;
mod geometry;
mod gl;
mod input;
mod main_loop;
mod text;

pub(crate) mod util;

pub mod draw;
pub mod math;

// Re-export dependencies that occur in our public API.
pub use glow;
pub use nalgebra;

pub use canvas::Canvas;
pub use color::{Color3, Color4};
pub use draw::{Batch, Texture};
pub use error::Error;
pub use input::{Event, InputState, Key};
pub use main_loop::main_loop;
pub use math::{Camera, Rect, RotatedRect, Screen};
pub use text::{Font, TextBatch};
