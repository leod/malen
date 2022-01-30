use nalgebra::Vector2;
use slab::Slab;

use super::{shape_shape_overlap, Overlap, Rect, Shape};

#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub key: usize,
    pub shape: Shape,
    pub data: T,
}

#[derive(Debug, Clone)]
struct Cell<T> {
    entries: Slab<Entry<T>>,
}

#[derive(Debug, Clone)]
struct Locations {
    cell_keys: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct Grid<T> {
    grid_rect: Rect,
    cell_size: f32,
    num_cells: Vector2<usize>,
    cells: Vec<Cell<T>>,
    locations: Slab<Locations>,
}

impl<T> Grid<T>
where
    T: Clone,
{
    pub fn new(grid_rect: Rect, cell_size: f32) -> Self {
        let num_cells_x = (grid_rect.size.x / cell_size).ceil() as usize;
        let num_cells_y = (grid_rect.size.y / cell_size).ceil() as usize;
        let cells = vec![
            Cell::<T> {
                entries: Slab::new()
            };
            num_cells_x * num_cells_y
        ];

        Self {
            grid_rect,
            cell_size,
            num_cells: Vector2::new(num_cells_x, num_cells_y),
            cells,
            locations: Slab::new(),
        }
    }

    pub fn insert(&mut self, shape: Shape, data: T) -> usize {
        let key = self.locations.insert(Locations {
            cell_keys: Vec::new(),
        });
        let mut cell_keys = Vec::new();

        let entry = Entry {
            key,
            shape: shape.clone(),
            data,
        };

        for cell_index in self.rasterize(&shape) {
            let data_key = self.cells[cell_index].entries.insert(entry.clone());
            cell_keys.push((cell_index, data_key));
        }

        self.locations.get_mut(key).unwrap().cell_keys = cell_keys;

        key
    }

    pub fn remove(&mut self, key: usize) {
        for (cell_index, data_key) in self.locations.remove(key).cell_keys {
            self.cells[cell_index].entries.remove(data_key);
        }
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.entries.clear();
        }
    }

    pub fn overlap<'a>(
        &'a self,
        shape: &'a Shape,
    ) -> impl Iterator<Item = (&'a Entry<T>, Overlap)> + 'a {
        self.rasterize(shape).flat_map(|cell_index| {
            self.cells[cell_index]
                .entries
                .iter()
                .filter_map(|(_, entry)| {
                    shape_shape_overlap(shape, &entry.shape).map(|overlap| (entry, overlap))
                })
        })
    }

    fn rasterize(&self, shape: &Shape) -> impl Iterator<Item = usize> {
        let mut rect = shape.bounding_rect();
        rect.center += self.grid_rect.size / 2.0;
        rect.center /= self.cell_size;
        rect.size /= self.cell_size;

        let num_cells = self.num_cells;

        let clip_x = |x: f32| (x.max(0.0) as usize).min(num_cells.x);
        let clip_y = |y: f32| (y.max(0.0) as usize).min(num_cells.y);

        let range_x = clip_x(rect.left_x().floor())..clip_x(rect.right_x().ceil());
        let range_y = clip_y(rect.top_y().floor())..clip_y(rect.bottom_y().ceil());

        range_x.flat_map(move |x| range_y.clone().map(move |y| y * num_cells.x + x))
    }
}
