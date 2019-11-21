#![allow(dead_code)]
extern crate ini;
extern crate rand;
extern crate rayon;
extern crate std;
use crate::ini::Ini;
use crate::std::env::args;
use crate::std::fs;
use std::io::ErrorKind;

mod particules;
mod tree;
mod write;
use crate::tree::*;
use crate::write::*;

fn simulation(tree: &mut Tree, time: f64, folder: String, crash_time: f64) {

    //create folders
    fs::create_dir(folder.clone()).unwrap_or_else(|err| {
        if err.kind() == ErrorKind::AlreadyExists {
            panic!("Simulation file already exists! Please change 'folder' in the configuration file.")
        } else {
            panic!("Unforeseen error creating simulation folder")
        }
    });
    fs::create_dir(format!("{}/positions", folder)).unwrap();
    fs::create_dir(format!("{}/densities", folder)).unwrap();

    //time
    let mut t = 0f64;
    //vector for infos
    let mut infos = Vec::new();
    //vector for inertia matrix
    let mut inertia_matrices = Vec::new();
    //count files
    let mut c = 0;
    //we save the general value of mu (dt = dynamical_time / mu)
    let mu = tree.mu;
    //and theta
    let theta = tree.theta;
    while t < time {
        //we use special values of theta and mu for the start of the simulation
        if t < crash_time {
            tree.mu = tree.mu_init;
            tree.theta = tree.theta_init;
        } else {
            tree.mu = mu;
            tree.theta = theta;
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

        //save those values in vectors
        infos.push(vec![
            t,
            tree.dynamical_time,
            tree.energy,
            tree.virial,
            tree.rayons[0],
            tree.rayons[1],
            tree.rayons[2],
        ]);
        inertia_matrices.push(tree.inertia_matrix);

        //write to file the positions of the particules and the density
        write_positions(&tree, format!("{}/positions/{}.csv", folder, c.to_string()));
        write_density(
            &tree,
            format!("{}/densities/{}.csv", folder.clone(), t.to_string()),
        );

        //simulate 10 steps
        for _ in 0..10 {
            //update the positions and velocity of all particules
            tree.leap_frog();
            //increment the current time, in dynamical time scale
            t += tree.dt / tree.dynamical_time;
        }

        c = c + 1;
    }

    //write all the values of infos and inertia_matrices to file
    write_infos(&infos, &inertia_matrices, folder.clone());
}

fn read<T>(section: &std::collections::HashMap<std::string::String, std::string::String>, expr: &str) -> T 
    where T: std::str::FromStr, <T as std::str::FromStr>::Err: std::fmt::Debug {
    section.get(expr).unwrap().parse().unwrap()
}

fn main() {
    //read values from the configuration file
    let arg: String = args().nth(1).unwrap();
    let conf = Ini::load_from_file(format!("./{}", arg)).unwrap();

    let section = conf.section(None::<String>).unwrap();
    
    //number of particules
    let nb_particules = read(section, "nb_particules");
    //number of particles positions saved
    let nb_particules_save = read(section, "nb_particules_save");
    //dt = dynamycal_time / mu
    let mu = read(section, "mu");
    //epsilon = (4/(3*N*pi))^(1/3) R50 / lambda
    let lambda = read(section, "lambda");
    //initial value of the virial ratio
    let virial: f64 = read(section, "virial");
    //duration of the simulation in dynamical time
    let time = read(section, "time");
    //approximation of the acceleration
    let theta = read(section, "theta");
    //is it a plummer model or a uniform sphere
    let plummer = read(section, "plummer");
    //number of bins used for the density
    let nb_bins = read(section, "nb_bins");
    //number of neighbors used for the local density
    let nb_neighbors = read(section, "nb_neighbors");
    //folder name
    let folder = section.get("folder").unwrap();
    //we use special theta and mu for the start of the simulation
    let crash_time = read(section, "crash_time");
    let mu_init = read(section, "mu_init");
    let theta_init = read(section, "theta_init");

    /*
    //number of particules
    let nb_particules = section.get("nb_particules").unwrap().parse().unwrap();
    //number of particles positions saved
    let nb_particules_save = section.get("nb_particules_save").unwrap().parse().unwrap();
    //dt = dynamycal_time / mu
    let mu = section.get("mu").unwrap().parse().unwrap();
    //epsilon = (4/(3*N*pi))^(1/3) R50 / lambda
    let lambda = section.get("lambda").unwrap().parse().unwrap();
    //initial value of the virial ratio
    let virial: f64 = section.get("virial").unwrap().parse().unwrap();
    //duration of the simulation in dynamical time
    let time = section.get("time").unwrap().parse().unwrap();
    //approximation of the acceleration
    let theta = section.get("theta").unwrap().parse().unwrap();
    //is it a plummer model or a uniform sphere
    let plummer = section.get("plummer").unwrap().parse().unwrap();
    //number of bins used for the density
    let nb_bins = section.get("nb_bins").unwrap().parse().unwrap();
    //number of neighbors used for the local density
    let nb_neighbors = section.get("nb_neighbors").unwrap().parse().unwrap();
    //folder name
    let folder = section.get("folder").unwrap();
    //we use special theta and mu for the start of the simulation
    let crash_time = section.get("crash_time").unwrap().parse().unwrap();
    let mu_init = section.get("mu_init").unwrap().parse().unwrap();
    let theta_init = section.get("theta_init").unwrap().parse().unwrap();
    */

    //build the octree and generate particules
    let mut tree = Tree::new_tree(
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

    //run the simulation
    simulation(&mut tree, time, folder.to_string(), crash_time);
}
