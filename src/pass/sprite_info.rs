use std::collections::BTreeMap;

use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::Vector2;

use crate::gl::{self, Texture, UniformBlock, UniformBuffer};

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct SpriteInfoBlock {
    size: Vector2<f32>,
}

pub struct SpriteInfos(BTreeMap<glow::Texture, UniformBuffer<SpriteInfoBlock>>);

impl SpriteInfos {
    pub fn new() -> Self {
        SpriteInfos(BTreeMap::new())
    }

    pub fn get(&mut self, texture: &Texture) -> Result<&UniformBuffer<SpriteInfoBlock>, gl::Error> {
        if !self.0.contains_key(&texture.id()) {
            // TODO: Fails if OpenGL reuses ids -- need to introduce our own unique IDs
            // TODO: Max size for sprite info cache
            let buffer = UniformBuffer::new(
                texture.gl(),
                SpriteInfoBlock {
                    size: texture.size().cast::<f32>(),
                },
            )?;
            self.0.insert(texture.id(), buffer);
        }

        Ok(self.0.get(&texture.id()).unwrap())
    }
}

impl UniformBlock for SpriteInfoBlock {}
