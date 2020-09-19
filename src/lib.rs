//! `webglee` is yet another one of these libraries for 2D web game
//! development.

mod context;
pub mod geom;
mod input;
mod main_loop;

pub use golem;
pub use golem::glow;

pub use context::{Context, Error};
pub use input::{Event, Input, KeysState, VirtualKeyCode};
pub use main_loop::main_loop;

pub use geom::{Matrix2, Matrix3, Point2, Vector2};
