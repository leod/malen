use std::collections::{btree_map, BTreeMap};

use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::Vector2;

use crate::gl::{self, Texture, Uniform, UniformBlock};

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct SpriteInfoBlock {
    size: Vector2<f32>,
}

impl UniformBlock for SpriteInfoBlock {}

pub struct SpriteInfos(BTreeMap<glow::Texture, Uniform<SpriteInfoBlock>>);

impl SpriteInfos {
    pub fn new() -> Self {
        SpriteInfos(BTreeMap::new())
    }

    pub fn get(&mut self, texture: &Texture) -> Result<&Uniform<SpriteInfoBlock>, gl::Error> {
        if let btree_map::Entry::Vacant(entry) = self.0.entry(texture.id()) {
            // TODO: Fails if OpenGL reuses ids -- need to introduce our own unique IDs
            // TODO: Max size for sprite info cache
            let buffer = Uniform::new(
                texture.gl(),
                SpriteInfoBlock {
                    size: texture.size().cast::<f32>(),
                },
            )?;

            entry.insert(buffer);
        }

        Ok(self.0.get(&texture.id()).unwrap())
    }
}
