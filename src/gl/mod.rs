#[macro_use]
mod vertex;

mod blend;
mod context;
mod depth_test;
mod draw;
mod draw_params;
mod draw_timer;
mod draw_unit;
mod element_buffer;
mod error;
mod framebuffer;
mod program;
mod texture;
mod uniform;
mod uniform_block;
mod vertex_array;
mod vertex_buffer;

pub use blend::{Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp};
pub use context::Context;
pub use depth_test::{DepthFunc, DepthTest};
pub use draw::{
    clear_color, clear_color_and_depth, clear_depth, draw, draw_instanced, with_framebuffer,
};
pub use draw_params::DrawParams;
pub use draw_timer::{DrawTimer, DrawTimingInfo};
pub use draw_unit::{DrawUnit, InstancedDrawUnit, PrimitiveMode};
pub use element_buffer::{Element, ElementBuffer};
pub use error::Error;
pub use framebuffer::{Framebuffer, NewFramebufferError};
pub use program::{Program, ProgramDef};
pub use texture::{
    LoadTextureError, NewTextureError, Texture, TextureMagFilter, TextureMinFilter, TextureParams,
    TextureValueType, TextureWrap,
};
pub use uniform::Uniform;
pub use uniform_block::{UniformBlock, UniformBlockDecls};
pub use vertex::{Attribute, AttributeValueType, DataType, Vertex, VertexDecls};
pub use vertex_array::VertexArray;
pub use vertex_buffer::VertexBuffer;

pub use crate::attributes;
