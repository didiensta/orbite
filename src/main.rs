//! # Orbite
//! N-body simulation of a globular cluster using a tree code.
//! Read the [README](README.md) to see how to run.

#![allow(dead_code)]
use ini::Ini;
use std::fs::File;

pub mod bins; // 'bin' is a reserved keyword, binaries of binS are not standalone
pub mod lib;
pub mod utils;

use lib::{tree::Tree, write};
use utils::io;

//Handles the whole simulation.
fn simulation(tree: &mut Tree, time: f64, mut file: &mut File, crash_time: f64, ser_fmt: usize) {
    //Current time
    let mut t = 0f64;
    //Iteration incrementor
    let mut c = 0usize;

    //main loop
    while t < time {
        //Use special values of theta and mu for the start of the simulation
        if t < crash_time {
            tree.mu = tree.mu_init;
            tree.theta = tree.theta_init;
        }

        println!("***");
        println!("t : {}", t);
        println!(" epsilon : {:?}", tree.epsilon);
        println!(" virial : {:?}", tree.virial);
        println!(" energy : {:?}", tree.energy);

        //compute new values
        tree.compute_center();
        tree.compute_rayons();
        tree.compute_inertia_matrix();
        tree.compute_energy();
        tree.compute_epsilon();
        tree.compute_dt();

        //Write saved data to file
        write::write_data_to_file(t, c, tree, &mut file, ser_fmt);

        //simulate 10 steps
        for _ in 0..10 {
            //update the positions and velocity of all particules
            tree.leap_frog();
            //increment the current time, in dynamical time scale
            t += tree.dt / tree.dynamical_time;
        }

        c += 1;
    }
}

fn main() {
    /////////////////////////////////////////////
    // read values from the configuration file //
    /////////////////////////////////////////////
    let arg = io::get_conf_file();

    let conf = Ini::load_from_file(format!("./{}", arg)).unwrap();

    let section = conf.section(Some("Parameters".to_owned())).unwrap();

    //number of particules
    let nb_particules = io::read(section, "nb_particules");
    //number of particles positions saved
    let nb_particules_save = io::read(section, "nb_particules_save");
    //dt = dynamycal_time / mu
    let mu = io::read(section, "mu");
    //epsilon = (4/(3*N*pi))^(1/3) R50 / lambda
    let lambda = io::read(section, "lambda");
    //initial value of the virial ratio
    let virial: f64 = io::read(section, "virial");
    //duration of the simulation in dynamical time
    let time = io::read(section, "time");
    //approximation of the acceleration
    let theta = io::read(section, "theta");
    //is it a plummer model or a uniform sphere
    let plummer = io::read(section, "plummer");
    //number of bins used for the density
    let nb_bins = io::read(section, "nb_bins");
    //number of neighbors used for the local density
    let nb_neighbors = io::read(section, "nb_neighbors");
    //folder name
    let folder = section.get("folder").unwrap();
    //we use special theta and mu for the start of the simulation
    let crash_time = io::read(section, "crash_time");
    let mu_init = io::read(section, "mu_init");
    let theta_init = io::read(section, "theta_init");

    //////////////////////////////////////////////
    /////////////// create folders ///////////////
    //////////////////////////////////////////////

    let (mut file, ser_fmt) = io::create_sim_file(folder);

    //////////////////////////////////////////////
    // build the octree and generate particules //
    //////////////////////////////////////////////
    let mut tree = Tree::new(
        nb_particules,
        nb_particules_save,
        mu,
        lambda,
        virial,
        theta,
        plummer,
        nb_bins,
        nb_neighbors,
        mu_init,
        theta_init,
    );

    /////////////////////////////////////////////
    //////////// run the simulation /////////////
    /////////////////////////////////////////////
    println!(
        "
    -----------------------
    Starting the simulation
    -----------------------"
    );
    simulation(&mut tree, time, &mut file, crash_time, ser_fmt);
}
