use std::cell::Cell;

use nalgebra::Vector2;
use slab::Slab;

use super::{shape_shape_overlap, Overlap, Rect, Shape};

#[derive(Debug, Copy, Clone, Default)]
pub struct GridInfo {
    pub entries: usize,
    pub entities: usize,
    pub lookups: usize,
    pub lookup_cells: usize,
    pub lookup_entries: usize,
}

#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub key: usize,
    pub shape: Shape,
    pub data: T,
}

#[derive(Debug, Clone)]
struct GridCell<T> {
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
    cells: Vec<GridCell<T>>,
    locations: Slab<Locations>,
    info: Cell<GridInfo>,
}

impl<T> Grid<T>
where
    T: Clone,
{
    pub fn new(grid_rect: Rect, cell_size: f32) -> Self {
        let num_cells_x = (grid_rect.size.x / cell_size).ceil() as usize;
        let num_cells_y = (grid_rect.size.y / cell_size).ceil() as usize;
        let cells = vec![
            GridCell::<T> {
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
            info: Cell::new(GridInfo::default()),
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

            self.info.set(GridInfo {
                entries: self.info.get().entries + 1,
                ..self.info.get()
            });
        }

        self.locations.get_mut(key).unwrap().cell_keys = cell_keys;
        self.info.set(GridInfo {
            entities: self.info.get().entities + 1,
            ..self.info.get()
        });

        key
    }

    pub fn remove(&mut self, key: usize) {
        for (cell_index, data_key) in self.locations.remove(key).cell_keys {
            self.cells[cell_index].entries.remove(data_key);

            debug_assert!(self.info.get().entries > 0);
            self.info.set(GridInfo {
                entries: self.info.get().entries - 1,
                ..self.info.get()
            });
        }

        debug_assert!(self.info.get().entities > 0);
        self.info.set(GridInfo {
            entities: self.info.get().entities - 1,
            ..self.info.get()
        });
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.entries.clear();
        }
        self.locations.clear();

        self.info.set(GridInfo {
            entities: 0,
            entries: 0,
            ..self.info.get()
        });
    }

    pub fn overlap<'a>(
        &'a self,
        shape: &'a Shape,
    ) -> impl Iterator<Item = (&'a Entry<T>, Overlap)> + 'a {
        self.info.set(GridInfo {
            lookups: self.info.get().lookups + 1,
            ..self.info.get()
        });

        self.rasterize(shape).flat_map(|cell_index| {
            self.info.set(GridInfo {
                lookup_cells: self.info.get().lookup_cells + 1,
                ..self.info.get()
            });

            self.cells[cell_index]
                .entries
                .iter()
                .filter_map(|(_, entry)| {
                    self.info.set(GridInfo {
                        lookup_entries: self.info.get().lookup_entries + 1,
                        ..self.info.get()
                    });

                    shape_shape_overlap(shape, &entry.shape).map(|overlap| (entry, overlap))
                })
        })
    }

    pub fn info(&self) -> GridInfo {
        self.info.get()
    }

    pub fn reset_info_lookups(&mut self) {
        self.info.set(GridInfo {
            lookups: 0,
            lookup_cells: 0,
            lookup_entries: 0,
            ..self.info.get()
        });
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
