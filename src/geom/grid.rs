struct Cell<T> {
    data: T,
}

pub struct Grid<T> {
    cell_size: f32,
    grid_rect: Rect,
    num_cells: Vector2<usize>,
    cells: Vec<Cell>,
}

impl<T> Grid<T> {
    pub fn new(cell_size: f32, grid_rect: Rect) -> Self {
    }
}