//! # Orbite
//! N-body simulation of a globular cluster using a tree code.
//! Read the [README](README.md) to see how to run.

#![allow(dead_code)]
use ini::Ini;
use std::fs::File;

mod particules;
mod tree;
mod write;

use tree::Tree;
use write::write_data_to_file;

//Handles the whole simulation.
fn simulation(tree: &mut Tree, time: f64, mut file: &mut File, crash_time: f64) {
    //Current time
    let mut t = 0f64;
    //Iteration incrementor
    let mut c = 0usize;

    //main loop
    while t < time {
        //we use special values of theta and mu for the start of the simulation
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
        write_data_to_file(t, c, tree, &mut file);

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

pub mod io {
    //! Handles the I/O logic.

    use std::env::args;
    use std::fs::{create_dir, remove_dir_all};
    use std::io::{stdin, stdout, ErrorKind, Write};

    pub fn create_sim_folder(folder: &str) {
        create_dir(folder).unwrap_or_else(|err| {
            if err.kind() == ErrorKind::AlreadyExists {
                println!(
                    "Simulation folder already exists! \
                     Do you whish to erase it? (y/_)"
                );
                let user_input = get_user_input_from_stdout();
                if user_input == "y" {
                    println!("Sure? Everything will be lost! (y/_)");
                    let user_input = get_user_input_from_stdout();
                    if user_input == "y" {
                        println!("Erasing simulation folder...");
                        remove_dir_all("sim").unwrap();
                        create_dir("sim").unwrap();
                    } else {
                        println!("Exiting...");
                        std::process::exit(1);
                    }
                } else {
                    println!("Exiting...");
                    std::process::exit(1);
                }
            } else {
                panic!("Unforeseen error creating simulation folder.")
            }
        });
    }

    pub fn get_user_input_from_stdout() -> String {
        //! Gets and cleans the user input from stdout

        let _ = stdout().flush();
        let mut user_input = String::new();

        stdin()
            .read_line(&mut user_input)
            .expect("Couldn't read the file name");

        //Remove potential \r's and \n's
        loop {
            if let Some('\n') = user_input.chars().next_back() {
                user_input.pop();
                continue;
            } else if let Some('\r') = user_input.chars().next_back() {
                user_input.pop();
                continue;
            }
            break;
        }

        user_input
    }

    pub fn get_conf_file() -> String {
        //! Tries to get the configuration file

        if let Some(argument) = args().nth(1) {
            //If an argument is given, try it.
            argument
        } else {
            //Else, ask the user!
            println!("Please enter a configuration file name:");

            get_user_input_from_stdout()
        }
    }

    pub fn read<T>(
        section: &std::collections::HashMap<std::string::String, std::string::String>,
        expr: &str,
    ) -> T
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        //Yuck...
        //Tring to factorise code here...
        section.get(expr).unwrap().parse().unwrap()
    }
}

fn main() {
    /////////////////////////////////////////////
    // read values from the configuration file //
    /////////////////////////////////////////////
    let arg = io::get_conf_file();

    let conf = Ini::load_from_file(format!("./{}", arg)).unwrap();

    let section = conf.section(None::<String>).unwrap();

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

    io::create_sim_folder(folder);
    let mut file = File::create("sim/data").unwrap();

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
    println!("Starting the simulation");
    simulation(&mut tree, time, &mut file, crash_time);
}
