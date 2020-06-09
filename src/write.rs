use crate::tree::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;

pub fn write_positions(tree: &Tree, file_name: String) {
    let mut file = File::create(file_name).unwrap();
    for i in 0..tree.nb_save {
        write!(
            &mut file,
            "{};{};{};",
            tree.particules[i].position[0] - tree.center[0],
            tree.particules[i].position[1] - tree.center[1],
            tree.particules[i].position[2] - tree.center[2]
        )
        .unwrap();
        write!(
            &mut file,
            "{};{}\n",
            tree.particules[i].cinetic,
            tree.particules[i].mass * tree.particules[i].potential
        )
        .unwrap();
    }
}

// Added to the original code
pub fn write_velocities(tree: &Tree, file_name: String) {
    let mut file = File::create(file_name).unwrap();
    for i in 0..tree.nb_save {
        write!(
            &mut file,
            "{};{};{}\n",
            tree.particules[i].speed[0],
            tree.particules[i].speed[1],
            tree.particules[i].speed[2]
        )
        .unwrap();
    }
}

pub fn write_infos(infos: &Vec<Vec<f64>>, inertia_matrices: &Vec<[f64; 9]>, folder_name: String) {
    let mut file = File::create(format!("{}/infos.csv", folder_name)).unwrap();
    for info in infos.iter() {
        for i in info {
            write!(&mut file, "{};", i).unwrap();
        }
        write!(&mut file, "\n").unwrap();
    }
    let mut file_inertia = File::create(format!("{}/inertia_matrix.csv", folder_name)).unwrap();
    for matrix in inertia_matrices.iter() {
        for i in matrix {
            write!(&mut file_inertia, "{};", i).unwrap();
        }
        write!(&mut file_inertia, "\n").unwrap();
    }
}

//compute the density profile and then write it to file
pub fn write_density(tree: &Tree, file_name: String) {
    //compute and sort distances
    let mut distances: Vec<f64> = tree
        .particules
        .par_iter()
        .map(|p| {
            f64::sqrt(
                (0..3)
                    .into_iter()
                    .map(|i| (p.position[i] - tree.center[i]).powf(2f64))
                    .sum(),
            )
        })
        .collect();
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let number_of_bins = tree.nb_bins;
    let mut bins = vec![0f64; number_of_bins];
    let mut bins_radii = vec![0f64; number_of_bins];
    let bin_size = tree.particules.len() / number_of_bins;
    let mut last_radius = 0f64;

    //compute the density of each slices of bin_size particules
    for i in 0..number_of_bins - 1 {
        bins_radii[i] = distances[(i + 1) * bin_size];
        let volume =
            4. / 3. * std::f64::consts::PI * (bins_radii[i].powf(3f64) - last_radius.powf(3f64));
        bins[i] = bin_size as f64 / volume / tree.particules.len() as f64;
        last_radius = bins_radii[i];
    }

    let mut file = File::create(file_name).unwrap();
    for i in 0..number_of_bins - 1 {
        write!(&mut file, "{};{}\n", bins_radii[i], bins[i]).unwrap();
    }
}
