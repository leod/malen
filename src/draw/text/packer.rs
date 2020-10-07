//! Packing cached glyphs into a texture atlas.
//!
//! Heavily inspired by this:
//! https://github.com/17cupsofcoffee/tetra/blob/main/src/graphics/text/packer.rs

use golem::{ColorFormat, Texture, TextureFilter};

use crate::{Context, Error};

/// A shelf has a fixed height and grows in width as more glyphs are added.
#[derive(Clone, Debug)]
struct Shelf {
    /// The X position at which the next glyph will be inserted.
    next_x: usize,

    /// The fixed Y position of this shelf's top.
    top_y: usize,

    /// The fixed width of this shelf.
    width: usize,

    /// The fixed height of this shelf. Only glyphs that are at most this high
    /// can be added to this shelf.
    height: usize,
}

impl Shelf {
    pub fn allocation_costs(&self, space_width: usize, space_height: usize) -> Option<usize> {
        if self.next_x + space_width > self.width {
            // The space does not fit into this shelf horizontally.
            None
        } else if space_height > self.height {
            // The space does not fit into this shelf vertically.
            None
        } else {
            // The space fits into this shelf. The costs are higher if we waste
            // more vertical space.
            Some(self.height - space_height)
        }
    }
}

pub struct ShelfPacker {
    texture: Texture,
    shelves: Vec<Shelf>,
    next_y: usize,
}

impl ShelfPacker {
    pub fn new(ctx: &Context, width: usize, height: usize) -> Result<ShelfPacker, Error> {
        let mut texture = Texture::new(ctx.golem_context())?;
        texture.set_image(None, width as u32, height as u32, ColorFormat::RGBA);
        texture.set_magnification(TextureFilter::Nearest)?;
        texture.set_minification(TextureFilter::Nearest)?;

        Ok(ShelfPacker {
            texture,
            shelves: Vec::new(),
            next_y: 0,
        })
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn insert(&mut self, data: &[u8], width: usize, height: usize) -> Option<(usize, usize)> {
        let space = self.allocate_space(width, height);

        if let Some((x, y)) = space {
            self.texture.set_subimage(
                data,
                x as u32,
                y as u32,
                width as u32,
                height as u32,
                ColorFormat::RGBA,
            )
        }

        space
    }

    fn allocate_space(
        &mut self,
        space_width: usize,
        space_height: usize,
    ) -> Option<(usize, usize)> {
        let texture_width = self.texture.width() as usize;
        let texture_height = self.texture.height() as usize;

        let best_shelf = self
            .shelves
            .iter_mut()
            .filter_map(|shelf| {
                shelf
                    .allocation_costs(space_width, space_height)
                    .map(|costs| (costs, shelf))
            })
            .min_by_key(|(costs, _)| *costs);

        if let Some((_, best_shelf)) = best_shelf {
            // Use existing shelf
            let position = (best_shelf.next_x, best_shelf.top_y);
            best_shelf.next_x += space_width;
            Some(position)
        } else if self.next_y + space_height < texture_height {
            // Create a new shelf
            let position = (0, self.next_y);

            self.shelves.push(Shelf {
                next_x: space_width,
                top_y: self.next_y,
                width: texture_width,
                height: space_height,
            });

            self.next_y += texture_height;

            Some(position)
        } else {
            // We ran out of space
            None
        }
    }
}
