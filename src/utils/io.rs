//! Handles the I/O logic.

//use rmp_serde::Serializer;
use std::{
    env::args,
    fs::{create_dir, read_to_string, remove_dir_all, File},
    io::{stdin, stdout, ErrorKind, Write},
};

use crate::{lib::write::Data, Tree};

const MESSAGEPACK: usize = 1;
const CBOR: usize = 2;
const PICKLE: usize = 3;

pub fn save_counter_to_file(c: usize, folder: &str) {
    // Creates a file within
    let filename = format!("{:?}/counter.txt", folder);
    let mut file = File::create(filename).unwrap();
    writeln!(file, "{:?}", c).unwrap();
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
        MESSAGEPACK => format!("{:?}/data.msgpack", folder),
        CBOR => format!("{:?}/data.cbor", folder),
        PICKLE => format!("{:?}/data.pickle", folder),
        _ => panic!("Error while creating the data file!"), // This should not happen...
    };

    let file = File::create(filename).unwrap();
    (file, ser_fmt)
}

pub fn get_user_input_from_stdout() -> String {
    //! Get and clean the user input from stdout

    let _ = stdout().flush();
    let mut user_input = String::new();

    stdin()
        .read_line(&mut user_input)
        .expect("Couldn't read the file name");

    remove_end_characters(user_input)
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
- MessagePack\n- CBOR\n- Pickle\n- BSON\n- Protobuf"
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
        "messagepack" => Ok(MESSAGEPACK),
        "cbor" => Ok(CBOR),
        "pickle" => Ok(PICKLE),
        _ => Err(0),
    }
}

pub fn open_sim_data_file(sim_data_file_path: String) -> File {
    //! Return the simulation data as a File, retry if given path results in file not found, else panic
    let sim_data_file = File::open(sim_data_file_path).unwrap_or_else(|err| {
        if err.kind() == ErrorKind::NotFound {
            println!("Could not find the simulation data folder, please enter a valid path:");
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

pub fn read_sim_data(c: usize, sim_folder_path: &String) -> Vec<Data> {
    //! Read simulation data

    // Open sim data file
    let sim_file_path = sim_folder_path.to_owned() + "/data.cbor";
    let sim_data_file = open_sim_data_file(sim_file_path);

    // Deserialize it
    let mut sim_data_vec = Vec::new();
    for _ in 0..c {
        let sim_data: Data = serde_cbor::from_reader(&sim_data_file)
            .expect("Error while deserializing simulation data!");
        sim_data_vec.push(sim_data);
    }
    sim_data_vec
}

pub fn write_data_to_file(t: f64, c: usize, tree: &Tree, file: &mut File, ser_fmt: usize) {
    let data = Data::new(t, c, tree);

    match ser_fmt {
        /*  MESSAGEPACK => { // err w/ .serialize, tmp rm
            let mut buf = Vec::new();
            let encoded_data = data.serialize(&mut Serializer::new(&mut buf)).unwrap();
            rmp_serde::encode::write(file, &encoded_data)
                .expect("Error: could not write data to file!");
        } */
        CBOR => {
            serde_cbor::to_writer(file, &data).expect("Error: could not write data to file!");
        }
        PICKLE => {
            let encoded_data = serde_pickle::to_vec(&data, true).unwrap();
            file.write_all(&encoded_data)
                .expect("Error: could not write data to file!");
        }
        _ => panic!("No data written to file"),
    }
}
