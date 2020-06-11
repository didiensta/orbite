//! Handles the I/O logic.

//use rmp_serde::Serializer;
use std::{
    env::args,
    fs::{create_dir, read_to_string, remove_dir_all, File},
    io::{stdin, stdout, ErrorKind, Write},
};

use crate::{
    lib::{particules::InitialState, write::Data},
    Tree,
};

const CBOR: usize = 1;
const PICKLE: usize = 2;
const CSV: usize = 3;

const PLUMMER: usize = 0;
const UNIFORM: usize = 1;
const HENON: usize = 2;
const CUSTOM: usize = 3;

pub struct Configuration {
    pub nb: usize,                   //number of particules
    pub nb_save: usize,              //number of particles positions saved
    pub mu: f64,                     //dt = dynamycal_time / mu
    pub lambda: f64,                 //epsilon = (4/(3*N*pi))^(1/3) R50 / lambda
    pub virial: f64,                 //initial value of the virial ratio
    pub theta: f64,                  //approximation of the acceleration
    pub initial_state: InitialState, //method of generation of the initial conditions
    pub nb_bins: usize,              //number of bins used for the density
    pub nb_neighbors: usize,         //number of neighbors used for the local density
    pub mu_init: f64,                //special mu for the start of the simulation
    pub theta_init: f64,             //idem
}

pub fn read_config(
    section: &std::collections::HashMap<std::string::String, std::string::String>,
) -> Configuration {
    Configuration {
        nb: read(section, "nb_particules"),
        nb_save: read(section, "nb_particules_save"),
        mu: read(section, "mu"),
        lambda: read(section, "lambda"),
        virial: read(section, "virial"),
        theta: read(section, "theta"),
        initial_state: read_initial_state(section),
        nb_bins: read(section, "nb_bins"),
        nb_neighbors: read(section, "nb_neighbors"),
        mu_init: read(section, "mu_init"),
        theta_init: read(section, "theta_init"),
    }
}

pub fn read_initial_state(
    section: &std::collections::HashMap<std::string::String, std::string::String>,
) -> InitialState {
    let state: usize = read(section, "initial_state");
    match state {
        PLUMMER => InitialState::Plummer,
        UNIFORM => InitialState::Plummer,
        HENON => InitialState::Henon,
        CUSTOM => {
            let path = read(section, "custom_init_path");
            InitialState::Custom(path)
        }
        _ => panic!("Initial state unresolved!"),
    }
}

pub fn save_counter_to_file(c: usize, folder: &str) {
    let filename = format!("{}/counter.txt", folder);
    let mut file = File::create(filename).unwrap();
    write!(file, "{}", c).unwrap();
}

pub fn create_sim_file(folder: &str) -> (File, usize) {
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
                    remove_dir_all(folder).unwrap();
                    println!("Creating simulation folder...");
                    create_dir(folder).unwrap();
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

    let ser_fmt = get_serialization_format();

    let filename = match ser_fmt {
        CBOR => format!("{}/data.cbor", folder),
        PICKLE => format!("{}/data.pickle", folder),
        CSV => format!("{}/data.csv", folder),
        _ => panic!("Error while creating the data file!"), // This should not happen...
    };
    let file = File::create(filename).unwrap();
    (file, ser_fmt)
}

pub fn get_user_input_from_stdout() -> String {
    //! Get and clean the user input from stdin

    let _ = stdout().flush();
    let mut user_input = String::new();

    stdin()
        .read_line(&mut user_input)
        .expect("Couldn't read stdin");

    remove_end_characters(user_input)
}

pub fn borrowed_get_user_input_from_stdout(s: &mut String) -> &String {
    //! Get the user input from stdin

    let _ = stdout().flush();

    stdin().read_line(s).expect("Couldn't read stdin");

    s
}

fn remove_end_characters(mut s: String) -> String {
    //! Remove potential \r's and \n's at end of String
    loop {
        if let Some('\n') = s.chars().next_back() {
            s.pop();
            continue;
        } else if let Some('\r') = s.chars().next_back() {
            s.pop();
            continue;
        }
        break;
    }
    s
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

pub fn get_serialization_format() -> usize {
    //! Tries to get the data serialization format.
    //! Loops until found.

    let user_input: String;

    if let Some(arg) = args().nth(2) {
        //If an argument is given, try it.
        user_input = arg;
    } else {
        //Else, ask the user!
        println!(
            "Please enter a supported serialization format:
- CBOR\n- Pickle\n- CSV (only partial support for now)"
        );

        user_input = get_user_input_from_stdout()
    }

    match serialization_format_check(user_input) {
        Ok(x) => x,
        Err(_) => {
            println!("Error: could not recognize serialization format!");
            get_serialization_format()
        }
    }
}

fn serialization_format_check(s: String) -> Result<usize, usize> {
    let s = remove_end_characters(s);
    match s.to_lowercase().as_str() {
        "cbor" => Ok(CBOR),
        "pickle" => Ok(PICKLE),
        "csv" => Ok(CSV),
        _ => Err(0),
    }
}

pub fn open_sim_data_file(sim_data_file_path: String) -> File {
    //! Return the simulation data as a File, retry if given path results in file not found, else panic
    let sim_data_file = File::open(sim_data_file_path).unwrap_or_else(|err| {
        if err.kind() == ErrorKind::NotFound {
            println!("Could not find the simulation data file, please enter a valid path:");
            let sim_data_file_path = get_user_input_from_stdout();
            open_sim_data_file(sim_data_file_path)
        } else {
            panic!("Unforeseen error accessing the simulation data!")
        }
    });
    sim_data_file
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

pub fn read_nb_iter(sim_folder_path: &String) -> usize {
    let sim_counter_path = sim_folder_path.to_owned() + "/counter.txt";
    let contents = read_to_string(sim_counter_path).expect("Error reading the counter file!");
    let counter: usize = contents
        .parse()
        .expect("Error parsing the content of counter.txt to an integer!");
    counter
}

pub fn read_sim_data(nb_iter: usize, sim_folder_path: &String) -> Vec<Data> {
    //! Read simulation data

    /* Open sim data file
    let sim_file_path = sim_folder_path.to_owned() + "/data.cbor";
    let sim_data_file = open_sim_data_file(sim_file_path); */

    // Deserialize it
    let mut sim_data_vec = Vec::new();
    for i in 0..nb_iter {
        // Open sim data file
        let sim_file_path = format!("{}/data_{}.cbor", sim_folder_path.to_owned(), i);
        let sim_data_file = open_sim_data_file(sim_file_path);

        // Read its data and append it
        let sim_data: Data = serde_cbor::from_reader(&sim_data_file)
            .expect("Error while deserializing simulation data!");
        sim_data_vec.push(sim_data);
    }
    sim_data_vec
}

pub fn write_data_to_file(t: f64, c: usize, tree: &Tree, ser_fmt: usize) {
    let data = Data::new(t, c, tree);

    match ser_fmt {
        CBOR => {
            let file_path = format!("sim/data_{}.cbor", c);
            let file = File::create(file_path).expect("Error creating data file!");
            serde_cbor::to_writer(file, &data).expect("Error: could not write data to file!");
        }
        PICKLE => {
            let file_path = format!("sim/data_{}.pickle", c);
            let mut file = File::create(file_path).expect("Error creating data file!");
            let encoded_data = serde_pickle::to_vec(&data, true).unwrap();
            file.write_all(&encoded_data)
                .expect("Error: could not write data to file!");
        }
        CSV => {
            // only used for Blender viz for now, not all data is saved
            let file_path = format!("sim/data_{}.csv", c);
            let mut file = File::create(file_path).expect("Error creating data file!");
            write!(file, "{}, {}", data.t, data.c).unwrap();
            for i in 0..tree.nb_save {
                write!(
                    file,
                    ", {}, {}, {}",
                    data.positions[i][0], data.positions[i][1], data.positions[i][2]
                )
                .unwrap();
            }
        }
        _ => panic!("No data written to file"),
    }
}

pub fn run_data_viz(folder: &String) {
    if let Some(arg) = args().nth(3) {
        // If an argument is given, follow it.
        if arg == "y" {
            crate::bins::after_run_viz::main(Some(folder));
        } else if arg == "n" {
            println!("Exiting...");
            std::process::exit(1);
        }
    } else {
        // Else, ask the user!
        println!("Run 3D data visualization (only works with CBOR for now)? (y/n)");
        let user_input = get_user_input_from_stdout();
        if user_input == "y" {
            println!(
                "
--------------------------
Starting the visualization
--------------------------"
            );
            crate::bins::after_run_viz::main(Some(folder));
        } else {
            println!("Exiting...");
            std::process::exit(1);
        }
    }
}
