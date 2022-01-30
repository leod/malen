use nalgebra::{Point2, Vector2};
use slab::Slab;

use crate::{
    data::{Geometry, RotatedSprite, SpriteVertex, TriangleTag},
    geom::{Rect, RotatedRect},
    Color4,
};

#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Point2<f32>,
    pub angle: f32,
    pub depth: f32,
    pub vel: Vector2<f32>,
    pub rot: f32,
    pub size: Vector2<f32>,
    pub color: Color4,
    pub slowdown: f32,
    pub age_secs: f32,
    pub max_age_secs: f32,
}

impl Particle {
    pub fn rotated_rect(&self) -> RotatedRect {
        Rect {
            center: self.pos,
            size: self.size,
        }
        .rotate(self.angle - std::f32::consts::PI / 2.0)
    }
}

#[derive(Debug, Clone)]
pub struct Particles {
    texture_size: Vector2<f32>,
    particles: Slab<Particle>,
}

impl Particles {
    pub fn new(texture_size: Vector2<u32>) -> Self {
        Self {
            texture_size: texture_size.cast(),
            particles: Slab::new(),
        }
    }

    pub fn texture_size(&self) -> Vector2<f32> {
        self.texture_size
    }

    pub fn len(&self) -> usize {
        self.particles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn spawn(&mut self, particle: Particle) {
        self.particles.insert(particle);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter().map(|(_, p)| p)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Particle> {
        self.particles.iter_mut().map(|(_, p)| p)
    }

    pub fn update(&mut self, dt_secs: f32) {
        for (_, particle) in self.particles.iter_mut() {
            particle.pos += dt_secs * particle.vel;

            let speed = particle.vel.norm();
            let new_speed = (speed - dt_secs * particle.slowdown * speed).max(0.0);
            particle.vel = new_speed / speed * particle.vel;

            let speed_factor =
                (1.0 - particle.age_secs / particle.max_age_secs).powf(particle.slowdown);
            particle.angle += dt_secs * speed_factor * particle.rot;

            particle.age_secs += dt_secs;
        }

        self.particles
            .retain(|_, particle| particle.age_secs <= particle.max_age_secs);
    }
}

impl<'a> Geometry<TriangleTag> for &'a Particles {
    type Vertex = SpriteVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        for (_, particle) in self.particles.iter() {
            RotatedSprite {
                rect: particle.rotated_rect(),
                depth: particle.depth,
                tex_rect: Rect::from_top_left(Point2::origin(), self.texture_size),
                color: Color4::new(
                    particle.color.r,
                    particle.color.g,
                    particle.color.b,
                    (1.0 - particle.age_secs / particle.max_age_secs).powf(2.0),
                ),
            }
            .write(elements, vertices);
        }
    }
}
