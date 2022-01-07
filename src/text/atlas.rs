//! Packing cached glyphs into a texture atlas.
//!
//! Heavily inspired by this:
//! https://github.com/17cupsofcoffee/tetra/blob/main/src/graphics/text/packer.rs

use std::rc::Rc;

use nalgebra::{Point2, Vector2};

use crate::{
    gl::{
        self, NewTextureError, Texture, TextureMagFilter, TextureMinFilter, TextureParams,
        TextureValueType, TextureWrap,
    },
    Rect,
};

/// A shelf has a fixed height and grows in width as more glyphs are added.
#[derive(Clone, Debug)]
struct Shelf {
    /// The X position at which the next glyph will be inserted.
    next_x: u32,

    /// The fixed Y position of this shelf's top.
    top_y: u32,

    /// The fixed width of this shelf.
    width: u32,

    /// The fixed height of this shelf. Only glyphs that are at most this high
    /// can be added to this shelf.
    height: u32,
}

impl Shelf {
    pub fn allocation_costs(&self, space: Vector2<u32>) -> Option<u32> {
        if self.next_x + space.x > self.width {
            // The space does not fit into this shelf horizontally.
            None
        } else if space.y > self.height {
            // The space does not fit into this shelf vertically.
            None
        } else {
            // The space fits into this shelf. The costs are higher if we waste
            // more vertical space.
            Some(self.height - space.y)
        }
    }
}

pub struct Atlas {
    texture: Texture,
    shelves: Vec<Shelf>,
    next_y: u32,
}

impl Atlas {
    pub fn new(gl: Rc<gl::Context>, size: Vector2<u32>) -> Result<Atlas, NewTextureError> {
        let texture = Texture::new(
            gl,
            size,
            TextureParams {
                value_type: TextureValueType::RgbaU8,
                min_filter: TextureMinFilter::Nearest,
                mag_filter: TextureMagFilter::Nearest,
                wrap_vertical: TextureWrap::ClampToEdge,
                wrap_horizontal: TextureWrap::ClampToEdge,
            },
        )?;

        // Ugh, this is not nice, but suppresses Firefox WebGL warnings about
        // lazy texture initialization. Need to revisit this at some point.
        let mut zeros = Vec::new();
        zeros.resize((size.x * size.y * 4) as usize, 0);
        texture.set_sub_image(Point2::origin(), size, &zeros);

        Ok(Atlas {
            texture,
            shelves: Vec::new(),
            next_y: 0,
        })
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn insert(&mut self, data: &[u8], size: Vector2<u32>) -> Option<Rect> {
        assert!(size.x > 0);
        assert!(size.y > 0);

        let pos = self.allocate_space(size)?;

        self.texture.set_sub_image(pos, size, data);

        // Shift by half a pixel, so that the coords are in the center of
        // the texel.
        let tex_top_left = pos.cast::<f32>() + Vector2::new(0.5, 0.5);

        // I think we can think of the size as inclusive, so we need to
        // subtract one here.
        let tex_size = size.cast::<f32>() - Vector2::new(1.0, 1.0);

        Some(Rect::from_top_left(tex_top_left, tex_size))
    }

    fn allocate_space(&mut self, space: Vector2<u32>) -> Option<Point2<u32>> {
        let best_shelf = self
            .shelves
            .iter_mut()
            .filter_map(|shelf| shelf.allocation_costs(space).map(|costs| (costs, shelf)))
            .min_by_key(|(costs, _)| *costs);

        if let Some((_, best_shelf)) = best_shelf {
            // Use existing shelf
            let pos = Point2::new(best_shelf.next_x, best_shelf.top_y);
            best_shelf.next_x += space.x;
            Some(pos)
        } else if self.next_y + space.y < self.texture.size().y {
            // Create a new shelf
            let pos = Point2::new(0, self.next_y);

            self.shelves.push(Shelf {
                next_x: space.x,
                top_y: self.next_y,
                width: self.texture.size().x,
                height: space.y,
            });

            self.next_y += space.y;

            Some(pos)
        } else {
            // We ran out of space
            None
        }
    }
}
