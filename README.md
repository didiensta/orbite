# Orbite

## This is still a WIP.

## Install and run

Install Rust: https://www.rust-lang.org/tools/install

Use Cargo to build and run.

Build:

	cargo build --release

Run:

	./target/release/orbite configuration_file.ini

Or build and run at the same time:

	cargo run --release configuration_file.ini

## Configuration file

Use a configuration file to specify all the parameters of the simulation.
See conf.ini for an example.

## Plots

Plot the energy, virial, density...  with plot.py: 

    python plot.py folder_of_the_simulation

Analyse the orbits with periode.py:

    python periode.py folder_of_the_simulation 

## Videos

You can make a video of the simulation with gnuplot.sh:

    ./gnuplot.sh folder_of_the_simulation

It uses gnuplot to render an image at each saved time steps and then uses ffmpeg to animate them.
