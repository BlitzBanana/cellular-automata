#![deny(clippy::all)]
#![forbid(unsafe_code)]

use clap::Clap;
use log::error;
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod automata;

fn get_mouse_index(
    input: &mut WinitInputHelper,
    pixels: &mut Pixels,
    width: usize,
) -> Option<usize> {
    input
        .mouse()
        .and_then(|(mx, my)| winit::dpi::PhysicalPosition::new(mx, my).into())
        .and_then(|pos| pixels.window_pos_to_pixel((pos.x, pos.y)).ok())
        .map(|(x, y)| automata::Position { x, y }.to_index(width))
}

#[derive(Clap)]
#[clap(
    name = "Cellular Automata",
    version = "0.1.0",
    author = "Blitz <victor.rebiardcrepin@nutshell-lab.com>",
    about = "This program implements a basic cellular automata following Conway's Game of Life rules"
)]
struct Opts {
    #[clap(short, long, default_value = "150")]
    width: usize,

    #[clap(short, long, default_value = "100")]
    height: usize,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let opts: Opts = Opts::parse();
    let width = opts.width;
    let height = opts.height;

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(width as f64, width as f64);
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
        Pixels::new(width as u32, height as u32, surface_texture)?
    };
    let mut world = automata::World::new(width, height);

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::Space) {
                world.paused = !world.paused;
            }

            if input.key_pressed(VirtualKeyCode::E) {
                world = automata::World::new(width, height);
            }

            if input.mouse_held(0) {
                if let Some(index) = get_mouse_index(&mut input, &mut pixels, width) {
                    world.set_cell_state(index, automata::CellState::ALIVE);
                }
            }

            if input.mouse_held(1) {
                if let Some(index) = get_mouse_index(&mut input, &mut pixels, width) {
                    world.set_cell_state(index, automata::CellState::DEAD);
                }
            }

            if input.mouse_held(2) {
                if let Some(index) = get_mouse_index(&mut input, &mut pixels, width) {
                    world.set_cell_state(index, automata::CellState::IMMUTABLE);
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
