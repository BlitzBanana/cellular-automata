#![deny(clippy::all)]
#![forbid(unsafe_code)]

// Press space to pause/unpause
// Press mouse 1 to create a cell
// Press mouse 2 to kill a cell

use rayon::prelude::*;
use log::error;
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 300;
const HEIGHT: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn from_index(index: usize) -> Self {
        Position { x: index % WIDTH, y: index / WIDTH }
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
    ON,
    OFF,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    index: usize,
    position: Position,
    state: CellState,
}

struct World {
    cells: Vec<Cell>,
    paused: bool,
}

impl World {
    fn new() -> Self {
        let cells: Vec<Cell> = (0..(WIDTH * HEIGHT)).map(|index| {
            let position = Position::from_index(index);
            let state = CellState::OFF;

            Cell { index, position, state }
        }).collect();

        Self { cells, paused: true }
    }
  
    fn set_cell_state(&mut self, index: usize, state: CellState) {
        if self.cells.len() - 1 > index {
            self.cells[index].state = state;
        };
    }

    fn neighbours_indexes(&self, i: usize) -> Vec<usize> {
        let cell = self.cells[i];

        vec![
            cell.position.top(HEIGHT).left(WIDTH).to_index(WIDTH),
            cell.position.top(HEIGHT).to_index(WIDTH),
            cell.position.top(HEIGHT).right(WIDTH).to_index(WIDTH),
            cell.position.left(WIDTH).to_index(WIDTH),
            cell.position.right(WIDTH).to_index(WIDTH),
            cell.position.bottom(HEIGHT).left(WIDTH).to_index(WIDTH),
            cell.position.bottom(HEIGHT).to_index(WIDTH),
            cell.position.bottom(HEIGHT).right(WIDTH).to_index(WIDTH)
        ]
    }

    fn update(&mut self) {
        // Each cell should be able to guess it's next state alone to be run in parallel.
        let new_state: Vec<Cell> = self.cells.par_iter().map(|&cell| {
            match cell.state {
                CellState::IMMUTABLE => { cell }
                CellState::ON | CellState::OFF  => {
                    let neighbours_indexes = self.neighbours_indexes(cell.index);
                    let alive_neighbours = neighbours_indexes
                        .iter()
                        .map(|&index| self.cells[index])
                        .filter(|cell| cell.state == CellState::ON)
                        .count();

                    // Let's update cell state :D (conway's rules here)
                    let new_state = if alive_neighbours == 2 {
                        cell.state
                    } else if alive_neighbours == 3 {
                        CellState::ON
                    } else  {
                        CellState::OFF
                    };

                    Cell {
                        index: cell.index,
                        position: cell.position,
                        state: new_state
                    }
                }
            }
        }).collect();

        self.cells = new_state;
    }

    /// Draw the `World` state to the frame buffer.
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let cell = self.cells[i];
            let rgba: [u8; 4] = match cell.state {
                CellState::IMMUTABLE => [0x6F, 0x6F, 0x6F, 0xFF],
                CellState::ON => [0x0, 0x0, 0x0, 0xFF],
                CellState::OFF => [0xFF, 0xFF, 0xFF, 0xFF],
            };

            pixel.copy_from_slice(&rgba);
        }
    }
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
    let mut world = World::new();

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

            if input.mouse_held(0) {
                if let Some((mx, my)) = input.mouse() {
                    if let Some((px, py)) = pixels.window_pos_to_pixel(winit::dpi::PhysicalPosition::new(mx, my).into()).ok() {
                        let index = py * WIDTH as usize + px;
                        world.set_cell_state(index, CellState::ON);
                    }
                }
            }

            if input.mouse_held(1) {
                if let Some((mx, my)) = input.mouse() {
                    if let Some((px, py)) = pixels.window_pos_to_pixel(winit::dpi::PhysicalPosition::new(mx, my).into()).ok() {
                        let index = py * WIDTH as usize + px;
                        world.set_cell_state(index, CellState::OFF);
                    }
                }
            }

            if input.mouse_held(2) {
                if let Some((mx, my)) = input.mouse() {
                    if let Some((px, py)) = pixels.window_pos_to_pixel(winit::dpi::PhysicalPosition::new(mx, my).into()).ok() {
                        let index = py * WIDTH as usize + px;
                        world.set_cell_state(index, CellState::IMMUTABLE);
                    }
                }
            }

            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            if !world.paused {
                world.update();
            }

            window.request_redraw();
        }
    });
}
