use rand_distr::StandardNormal;
use std::io;
use csv;

use crate::rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct Particule {
    pub position: [f64; 3],
    pub speed: [f64; 3],
    pub acceleration: [f64; 3],
    // 0.5*m*v^2
    pub cinetic: f64,
    pub potential: f64,
    pub mass: f64,
}

//generate nb particules with the data
fn from_csv_gen(nb: usize) -> Vec<Particule> {
    println!("Initial conditions from .csv file");
    let mut rng = rand::thread_rng();
    let mut particules = Vec::with_capacity(nb);
    let mut x;
    let mut y;
    let mut z;
    let mut vx;
    let mut vy;
    let mut vz;
    let mut c = 0;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_reader(io::stdin());

    for result in rdr.records() {
        let record = result.ok().unwrap();
        x = record.get(0).unwrap().parse::<f64>().unwrap();
        y = record.get(1).unwrap().parse::<f64>().unwrap();
        z = record.get(2).unwrap().parse::<f64>().unwrap();
        vx = record.get(3).unwrap().parse::<f64>().unwrap();
        vy = record.get(4).unwrap().parse::<f64>().unwrap();
        vz = record.get(5).unwrap().parse::<f64>().unwrap();
        // println!("{}, {}, {}, {}, {}, {}",x,y,z,vx,vy,vz);
        particules.push(Particule {
            position: [x, y, z],
            speed: [vx, vy, vz],
            acceleration: [0., 0., 0.],
            cinetic: 0f64,
            potential: 0f64,
            mass: 1. / (nb as f64),
        });
        c = c + 1;
    }
    if c != nb {
        println!("WARNING! : Number of particules in configuration file does not match the number of line of the .csv file")
    }
    particules
}

//generate nb particules with uniform distribution of velocities and positions on the unit sphere
//all particules have the same mass = 1/nb
//speed is uniform and ||v|| < 1
fn unif_gen(nb: usize) -> Vec<Particule> {
    let mut rng = rand::thread_rng();
    let mut particules = Vec::with_capacity(nb);
    let mut x;
    let mut y;
    let mut z;
    let mut vx;
    let mut vy;
    let mut vz;
    for _ in 0..nb {
        loop {
            x = rng.gen_range(-1., 1.);
            y = rng.gen_range(-1., 1.);
            z = rng.gen_range(-1., 1.);
            if x * x + y * y + z * z < 1. {
                break;
            }
        }
        loop {
            vx = rng.gen_range(-1., 1.);
            vy = rng.gen_range(-1., 1.);
            vz = rng.gen_range(-1., 1.);
            if vx * vx + vy * vy + vz * vz < 1. {
                break;
            }
        }
        particules.push(Particule {
            position: [x, y, z],
            speed: [vx, vy, vz],
            acceleration: [0., 0., 0.],
            cinetic: 0f64,
            potential: 0f64,
            mass: 1. / (nb as f64),
        });
    }
    particules
}

//UNTESTED
//return the density of the isochrone potential
fn henon_density(r: f64, b: f64) -> f64 {
    let a = (b * b + r * r).sqrt();
    let num = 3. * (b + a) * a * a - r * r * (b + 3. * a);
    let denum = 4. * std::f64::consts::PI * (b + a).powf(3.) + a.powf(3.);
    return num / denum;
}

//UNTESTED
//generate nb particules with isochrone potential
//use rejection sampling
//see https://en.wikipedia.org/wiki/Rejection_sampling
//speed ~ Gaussian
fn henon_gen(nb: usize) -> Vec<Particule> {
    let mut rng = rand::thread_rng();
    let mut particules = Vec::with_capacity(nb);
    let c = 10f64;
    let b = 1.5f64;
    let bondary = 10f64;
    for _ in 0..nb {
        let mut r;
        let mut u;
        loop {
            r = rng.gen_range(-bondary, bondary);
            u = rng.gen_range(0f64, 1f64);
            if u < 2. * bondary * henon_density(r, b) / c {
                break;
            }
        }

        let x1 = rng.gen_range(0f64, 1f64);
        let x2 = rng.gen_range(0f64, 1f64);

        let z = (1. - 2. * x1) * r;
        let x = (r * r - z * z).sqrt() * (2. * std::f64::consts::PI * x2).cos();
        let y = (r * r - z * z).sqrt() * (2. * std::f64::consts::PI * x2).sin();

        particules.push(Particule {
            position: [x, y, z],
            speed: [
                rng.sample(StandardNormal),
                rng.sample(StandardNormal),
                rng.sample(StandardNormal),
            ],
            acceleration: [0., 0., 0.],
            cinetic: 0f64,
            potential: 0f64,
            mass: 1. / (nb as f64),
        });
    }
    particules
}

//generate a Plummer
fn plummer(nb: usize) -> Vec<Particule> {
    let mut rng = rand::thread_rng();
    let mut particules = Vec::with_capacity(nb);
    for _ in 0..nb {
        let x1 = rng.gen_range(0., 1.);
        let r = ((0.99f64 * x1).powf(-2. / 3.) - 1f64).powf(-1. / 2.);
        let x2 = rng.gen_range(0., 1.);
        let x3 = rng.gen_range(0., 1.);

        let z = (1f64 - 2f64 * x2) * r;
        let x = f64::sqrt(r * r - z * z) * f64::cos(2f64 * std::f64::consts::PI * x3);
        let y = f64::sqrt(r * r - z * z) * f64::sin(2f64 * std::f64::consts::PI * x3);

        let s_e = f64::sqrt(2f64) * (1f64 + r * r).powf(-1. / 4.);
        let mut x4;
        let mut x5;
        loop {
            x4 = rng.gen_range(0., 1.);
            x5 = rng.gen_range(0., 1.);
            if 0.1 * x5 < x4 * x4 * (1f64 - x4 * x4).powf(7. / 2.) {
                break;
            }
        }

        let q = x4;
        let v = q * s_e;

        let x6 = rng.gen_range(0., 1.);
        let x7 = rng.gen_range(0., 1.);

        let w = (1f64 - 2f64 * x6) * v;
        let u = f64::sqrt(v * v - w * w) * f64::cos(2f64 * std::f64::consts::PI * x7);
        let uu = f64::sqrt(v * v - w * w) * f64::sin(2f64 * std::f64::consts::PI * x7);

        particules.push(Particule {
            position: [x, y, z],
            speed: [w, u, uu],
            acceleration: [0f64, 0f64, 0f64],
            cinetic: 0f64,
            potential: 0f64,
            mass: 1. / (nb as f64),
        });
    }
    particules
}

pub fn generation(nb: usize, is_plummer: bool, from_csv: bool) -> Vec<Particule> {
    let particules;
    if from_csv {
        particules = from_csv_gen(nb);
    } else {
        if is_plummer {
            particules = plummer(nb);
        } else {
            particules = unif_gen(nb);
            //particules = henon_gen(nb);
        }
    }
    return particules;
}
