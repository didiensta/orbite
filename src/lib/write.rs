use crate::tree::Tree;
use rmp_serde::Serializer;
use serde::Serialize;
use std::fs::File;
use std::io::Write;

const MESSAGEPACK: usize = 1;
const CBOR: usize = 2;
const PICKLE: usize = 3;

#[derive(Serialize)]
struct Data {
    t: f64,
    c: usize,
    positions: Vec<[f64; 3]>,
    speeds: Vec<[f64; 3]>,
    rayons: [f64; 3],
    inertia_matrix: [f64; 9],
    energy: f64,
    virial: f64,
    dynamical_time: f64,
    espilon: f64,
}

impl Data {
    pub fn new(t: f64, c: usize, tree: &Tree) -> Data {
        //! Create a new Data instance.
        //! t is the current time, c the current iteration number,
        //! and all other relevant data is *copied* from tree.

        let mut positions = Vec::new();
        let mut speeds = Vec::new();

        for i in 0..tree.nb_save {
            positions.push([
                tree.particules[i].position[0] - tree.center[0],
                tree.particules[i].position[1] - tree.center[1],
                tree.particules[i].position[2] - tree.center[2],
            ]);

            speeds.push([
                tree.particules[i].speed[0],
                tree.particules[i].speed[1],
                tree.particules[i].speed[2],
            ])
        }
        Data {
            t,
            c,
            positions,
            speeds,
            rayons: tree.rayons,
            inertia_matrix: tree.inertia_matrix,
            energy: tree.energy,
            virial: tree.virial,
            dynamical_time: tree.dynamical_time,
            espilon: tree.epsilon,
        }
    }
}

pub fn write_data_to_file(t: f64, c: usize, tree: &Tree, file: &mut File, ser_fmt: usize) {
    let data = Data::new(t, c, tree);

    match ser_fmt {
        MESSAGEPACK => {
            let mut buf = Vec::new();
            let encoded_data = data.serialize(&mut Serializer::new(&mut buf)).unwrap();
            rmp_serde::encode::write(file, &encoded_data)
                .expect("Error: could not write data to file!");
        }
        CBOR => {
            serde_cbor::to_writer(file, &data).expect("Error: could not write data to file!");
        }
        PICKLE => {
            let encoded_data = serde_pickle::to_vec(&data, true).unwrap();
            file.write_all(&encoded_data)
                .expect("Error: could not write data to file!");
        }
        _ => println!("No data written to file"),
    }
}
