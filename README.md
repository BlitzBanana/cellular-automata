# Rust - Cellular Automata <img src=".github/ferris.png" title="Cellular Ferris" width="30" height="22">

> I'm a rust noob and i'm doing this kind of project to learn.

Basic [Conway's game of life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) rust implementation.

- 1 neighbor & less -> die (underpolupation)
- 2 neighbours -> keep on
- 3 neighbours -> born
- 4 neighbours & more -> die (overpolupation)

Computations are parallelized using the crate [rayon](https://crates.io/crates/rayon).  
Rendering done using the crate [pixels](https://crates.io/crates/pixels) ans [winit](https://crates.io/crates/winit).

## Run

```sh
git clone git@github.com:BlitzBanana/cellular-automata.git
cd cellular-automata
cargo run --release -- -w 250 -h 200
```

<h1 align="center">
	<img src=".github/preview.gif" title="Cellular Automata preview">
</h1>

## Keybindings

- Press `space` to pause/unpause.
- Press `mouse left` to spawn a cell.
- Press `mouse right` to kill a cell.
- Press `e` to erase the world.
