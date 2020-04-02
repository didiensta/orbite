//! Handles the I/O logic.

use std::env::args;
use std::fs::{create_dir, remove_dir_all, File};
use std::io::{stdin, stdout, ErrorKind, Write};

const MESSAGEPACK: usize = 1;
const CBOR: usize = 2;
const PICKLE: usize = 3;

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

    let ser_fmt = get_serialization_format();

    let file = match ser_fmt {
        MESSAGEPACK => File::create("sim/data.msgpack").unwrap(),
        CBOR => File::create("sim/data.cbor").unwrap(),
        PICKLE => File::create("sim/data.pickle").unwrap(),
        _ => panic!("Error while creating the data file!"), // This should not happen...
    };
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
