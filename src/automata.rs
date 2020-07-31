use rayon::prelude::*;

pub mod utils {
    pub fn coords_to_index(x: usize, y: usize, width: usize) -> usize {
        y * width + x
    }

    pub fn index_to_coords(index: usize, width: usize) -> (usize, usize) {
        (index % width, index / width)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    ALIVE,
    DEAD,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn to_index(&self, width: usize) -> usize {
        utils::coords_to_index(self.x, self.y, width)
    }

    fn from_index(index: usize, width: usize) -> Self {
        let (x, y) = utils::index_to_coords(index, width);
        Self { x, y }
    }

    fn left(&self, width: usize) -> Self {
        let x = self.x.checked_sub(1).unwrap_or(width - 1);
        Self { x, y: self.y }
    }

    fn right(&self, width: usize) -> Self {
        let x = self.x.checked_add(1).filter(|&v| v < width).unwrap_or(0);
        Self { x, y: self.y }
    }

    fn top(&self, height: usize) -> Self {
        let y = self.y.checked_sub(1).unwrap_or(height - 1);
        Self { x: self.x, y }
    }

    fn bottom(&self, height: usize) -> Self {
        let y = self.y.checked_add(1).filter(|&v| v < height).unwrap_or(0);
        Self { x: self.x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    index: usize,
    position: Position,
    state: State,
    neighbours_indexes: [usize; 8],
}

pub struct World {
    pub paused: bool,
    cells: Vec<Cell>,
}

fn neighbours_indexes(i: usize, width: usize, height: usize) -> [usize; 8] {
    let pos = Position::from_index(i, width);

    [
        pos.top(height).left(width).to_index(width),
        pos.top(height).to_index(width),
        pos.top(height).right(width).to_index(width),
        pos.left(width).to_index(width),
        pos.right(width).to_index(width),
        pos.bottom(height).left(width).to_index(width),
        pos.bottom(height).to_index(width),
        pos.bottom(height).right(width).to_index(width),
    ]
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            paused: true,
            cells: (0..(width * height))
                .map(|index| Cell {
                    index,
                    position: Position::from_index(index, width),
                    state: State::DEAD,
                    neighbours_indexes: neighbours_indexes(index, width, height),
                })
                .collect(),
        }
    }

    pub fn set_cell_state(&mut self, index: usize, state: State) {
        if let Some(cell) = self.cells.get_mut(index) {
            cell.state = state
        };
    }

    pub fn update(&mut self) {
        if self.paused {
            return;
        }

        // A cell cannot mutate other cells, only itself
        // This allows us to run the update in parallel (using rayon crate here)
        let new_state: Vec<Cell> = self
            .cells
            .par_iter()
            .map(|&cell| {
                let alive_neighbours = cell
                    .neighbours_indexes
                    .iter()
                    .map(|&index| self.cells[index])
                    .filter(|cell| cell.state == State::ALIVE)
                    .count();

                let state = match alive_neighbours {
                    3 => State::ALIVE,
                    2 => cell.state,
                    _ => State::DEAD,
                };

                Cell { state, ..cell }
            })
            .collect();

        self.cells = new_state;
    }

    /// Draw the `World` state to the frame buffer.
    pub fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba: [u8; 4] = match self.cells[i].state {
                State::ALIVE => [0x1E, 0x1E, 0x1E, 0xFF],
                State::DEAD => [0xF8, 0xF8, 0xF8, 0xF8],
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
