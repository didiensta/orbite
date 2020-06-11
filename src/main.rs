//! # Orbite
//! N-body simulation of a globular cluster using a tree code.
//! Read the [README](README.md) to see how to run.

use ini::Ini;

pub mod bins; // 'bin' is a reserved keyword, binaries of binS are not standalone
pub mod lib;
pub mod utils;

use lib::tree::Tree;
use utils::io;

//Handles the whole simulation.
fn simulation(tree: &mut Tree, time: f64, crash_time: f64, ser_fmt: usize) -> usize {
    //Iteration incrementor
    let mut c = 0usize;

    //Current time
    let mut t = 0f64;

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
        io::write_data_to_file(t, c, tree, ser_fmt);

        //simulate 10 steps
        for _ in 0..10 {
            //update the positions and velocity of all particules
            tree.leap_frog();
            //increment the current time, in dynamical time scale
            t += tree.dt / tree.dynamical_time;
        }

        c += 1;
    }
    c
}

fn main() {
    /////////////////////////////////////////////
    // read values from the configuration file //
    /////////////////////////////////////////////
    let arg = io::get_conf_file();
    let conf = Ini::load_from_file(format!("./{}", arg)).unwrap(); //panics if orbite not called at crates's root
    let section = conf.section(Some("Parameters".to_owned())).unwrap();
    let time = io::read(section, "time"); //duration of the simulation in dynamical time
    let folder = section.get("folder").unwrap();
    let crash_time = io::read(section, "crash_time");
    let config = io::read_config(section);

    //////////////////////////////////////////////
    /////////////// create folders ///////////////
    //////////////////////////////////////////////
    let (_, ser_fmt) = io::create_sim_file(folder);

    //////////////////////////////////////////////
    // build the octree and generate particules //
    //////////////////////////////////////////////
    let mut tree = Tree::new(config);

    /////////////////////////////////////////////
    //////////// run the simulation /////////////
    /////////////////////////////////////////////
    println!(
        "
    -----------------------
    Starting the simulation
    -----------------------"
    );
    let c = simulation(&mut tree, time, crash_time, ser_fmt);

    io::save_counter_to_file(c, folder);

    io::run_data_viz(folder)
}
