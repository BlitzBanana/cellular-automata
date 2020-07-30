#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use rayon::prelude::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 150;
const HEIGHT: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn from_index(index: usize, width: usize) -> Self {
        Position {
            x: index % width,
            y: index / width,
        }
    }

    fn to_index(&self, width: usize) -> usize {
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
enum CellState {
    IMMUTABLE,
    ALIVE,
    DEAD,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    index: usize,
    position: Position,
    state: CellState,
}

struct World {
    width: usize,
    height: usize,
    paused: bool,
    cells: Vec<Cell>,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        let cells: Vec<Cell> = (0..(width * height))
            .map(|index| {
                let position = Position::from_index(index, width);
                let state = CellState::DEAD;

                Cell {
                    index,
                    position,
                    state,
                }
            })
            .collect();

        Self {
            width,
            height,
            cells,
            paused: true,
        }
    }

    fn set_cell_state(&mut self, index: usize, state: CellState) {
        if let Some(cell) = self.cells.get_mut(index) {
            cell.state = state
        };
    }

    fn neighbours_indexes(&self, i: usize) -> [usize; 8] {
        let (width, height) = (self.width, self.height);
        let cell = self.cells[i];

        [
            cell.position.top(height).left(width).to_index(width),
            cell.position.top(height).to_index(width),
            cell.position.top(height).right(width).to_index(width),
            cell.position.left(width).to_index(width),
            cell.position.right(width).to_index(width),
            cell.position.bottom(height).left(width).to_index(width),
            cell.position.bottom(height).to_index(width),
            cell.position.bottom(height).right(width).to_index(width),
        ]
    }

    fn update(&mut self) {
        if self.paused { return }

        // A cell cannot mutate other cells, only itself
        // This allows us to run the update in parallel (using rayon crate here)
        let new_state: Vec<Cell> = self
            .cells
            .par_iter()
            .map(|&cell| {
                match cell.state {
                    CellState::IMMUTABLE => cell,
                    CellState::ALIVE | CellState::DEAD => {
                        let neighbours_indexes = self.neighbours_indexes(cell.index);
                        let alive_neighbours = neighbours_indexes
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
                            index: cell.index,
                            position: cell.position,
                            state: new_state,
                        }
                    }
                }
            })
            .collect();

        self.cells = new_state;
    }

    /// Draw the `World` state to the frame buffer.
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let cell = self.cells[i];
            let rgba: [u8; 4] = match cell.state {
                CellState::IMMUTABLE => [0xFF, 0x0, 0x4D, 0xFF],
                CellState::ALIVE => [0x1E, 0x1E, 0x1E, 0xFF],
                CellState::DEAD => [0xF8, 0xF8, 0xF8, 0xF8],
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}

fn get_mouse_index(input: &mut WinitInputHelper, pixels: &mut Pixels, width: usize) -> Option<usize> {
    input
        .mouse()
        .and_then(|(mx, my)| winit::dpi::PhysicalPosition::new(mx, my).into())
        .and_then(|pos| pixels.window_pos_to_pixel((pos.x, pos.y)).ok())
        .and_then(|(x, y)| Some(Position { x, y }.to_index(width)))
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Cellular Automata")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, surface);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };
    let mut world = World::new(WIDTH, HEIGHT);

    event_loop.run(move |event, _, control_water_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_water_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_water_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::Space) {
                world.paused = !world.paused;
            }

            if input.key_pressed(VirtualKeyCode::E) {
                world = World::new(WIDTH, HEIGHT);
            }

            if input.mouse_held(0) {
                if let Some(index) = get_mouse_index(&mut input, &mut pixels, WIDTH) {
                    world.set_cell_state(index, CellState::ALIVE);
                }
            }

            if input.mouse_held(1) {
                if let Some(index) = get_mouse_index(&mut input, &mut pixels, WIDTH) {
                    world.set_cell_state(index, CellState::DEAD);
                }
            }

            if input.mouse_held(2) {
                if let Some(index) = get_mouse_index(&mut input, &mut pixels, WIDTH) {
                    world.set_cell_state(index, CellState::IMMUTABLE);
                }
            }

            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            world.update();
            window.request_redraw();
        }
    });
}
