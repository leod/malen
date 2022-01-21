mod bindings;

pub mod compose;
pub mod compose_with_indirect;
pub mod geometry_color;
pub mod geometry_color_sprite;
pub mod geometry_color_sprite_with_normals;
pub mod screen_light;
pub mod shaded_color;
pub mod shaded_color_sprite;
pub mod shaded_sprite;
pub mod shadow_map;

pub use bindings::{GLOBAL_LIGHT_PARAMS_BLOCK_BINDING, OBJECT_LIGHT_PARAMS_BLOCK_BINDING};
