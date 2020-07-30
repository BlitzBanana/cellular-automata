use rayon::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    fn from_index(index: usize, width: usize) -> Self {
        Position {
            x: index % width,
            y: index / width,
        }
    }

    pub fn to_index(&self, width: usize) -> usize {
        self.y * width + self.x
    }

    fn left(&self, width: usize) -> Position {
        let x = self.x.checked_sub(1).unwrap_or(width - 1);
        Position { x, y: self.y }
    }

    fn right(&self, width: usize) -> Position {
        let x = self.x.checked_add(1).filter(|&v| v < width).unwrap_or(0);
        Position { x, y: self.y }
    }

    fn top(&self, height: usize) -> Position {
        let y = self.y.checked_sub(1).unwrap_or(height - 1);
        Position { x: self.x, y }
    }

    fn bottom(&self, height: usize) -> Position {
        let y = self.y.checked_add(1).filter(|&v| v < height).unwrap_or(0);
        Position { x: self.x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    IMMUTABLE,
    ALIVE,
    DEAD,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    index: usize,
    position: Position,
    state: CellState,
    neighbours_indexes: [usize; 8],
}

pub struct World {
    pub width: usize,
    pub height: usize,
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
        let cells: Vec<Cell> = (0..(width * height))
            .map(|index| Cell {
                index,
                position: Position::from_index(index, width),
                state: CellState::DEAD,
                neighbours_indexes: neighbours_indexes(index, width, height),
            })
            .collect();

        Self {
            width,
            height,
            cells,
            paused: true,
        }
    }

    pub fn set_cell_state(&mut self, index: usize, state: CellState) {
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
                match cell.state {
                    CellState::IMMUTABLE => cell,
                    CellState::ALIVE | CellState::DEAD => {
                        let alive_neighbours = cell
                            .neighbours_indexes
                            .iter()
                            .map(|&index| self.cells[index])
                            .filter(|cell| cell.state == CellState::ALIVE)
                            .count();

                        // Let's update cell state :D (conway's rules here)
                        let new_state = if alive_neighbours == 2 {
                            cell.state
                        } else if alive_neighbours == 3 {
                            CellState::ALIVE
                        } else {
                            CellState::DEAD
                        };

                        Cell {
                            state: new_state,
                            ..cell
                        }
                    }
                }
            })
            .collect();

        self.cells = new_state;
    }

    /// Draw the `World` state to the frame buffer.
    pub fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba: [u8; 4] = match self.cells[i].state {
                CellState::IMMUTABLE => [0xFF, 0x0, 0x4D, 0xFF],
                CellState::ALIVE => [0x1E, 0x1E, 0x1E, 0xFF],
                CellState::DEAD => [0xF8, 0xF8, 0xF8, 0xF8],
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
