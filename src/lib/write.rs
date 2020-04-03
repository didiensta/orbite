use super::tree::Tree;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Data {
    t: f64,
    c: usize,
    pub positions: Vec<[f64; 3]>,
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
