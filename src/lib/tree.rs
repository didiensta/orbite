pub use super::particules::*;
use rayon::prelude::*;

pub struct Node {
    // 1/2 of the side of the box
    pub size: f64,
    pub center: [f64; 3],
    pub center_of_mass: [f64; 3],
    pub mass: f64,
    //id of the particule if there is one
    pub particule: Option<u32>,
    //vector with the id of the sub-nodes or None if the sub-node doesn't exist
    pub kids: [Option<u32>; 8],
}

impl Node {
    fn is_leaf(&self) -> bool {
        for kids in self.kids.iter() {
            if kids.is_some() {
                return false;
            }
        }
        true
    }
}

pub struct Tree {
    //vec of particules
    pub particules: Vec<Particule>,
    //vec of the nodes of the octree
    pub nodes: Vec<Node>,
    pub center: [f64; 3],
    //[R10, R50, R90]
    pub rayons: [f64; 3],
    pub inertia_matrix: [f64; 9],
    pub energy: f64,
    pub virial: f64,
    pub dynamical_time: f64,
    pub theta: f64,
    pub dt: f64,
    pub mu: f64,
    pub epsilon: f64,
    pub lambda: f64,
    pub nb_save: usize,
    pub nb_bins: usize,
    pub nb_neighbors: usize,
    pub mu_init: f64,
    pub theta_init: f64,
}

impl Tree {
    //return in which sub-node the particule is
    fn get_subtree_id(&self, node_id: usize, p_id: usize) -> usize {
        //we need 3 bites X Y Z to describe in which box the particule is

        //we set X = Y = Z = 0
        //id = (XYZ)b (in binary)
        let mut id = 0;

        if self.particules[p_id].position[0] > self.nodes[node_id].center[0] {
            //if particule.x > center.x
            //we set the X bit to one
            //X = 1
            //so we add 4, because 4 = (100)b
            id += 4;
        }
        if self.particules[p_id].position[1] > self.nodes[node_id].center[1] {
            //Y = 1
            id += 2;
        }
        if self.particules[p_id].position[2] > self.nodes[node_id].center[2] {
            //Z = 1
            id += 1;
        }
        id
    }

    //if needed make a sub-node to the node mother_id, so it can contains the particule p_id
    fn make_kid(&mut self, mother_id: usize, p_id: usize) {
        let subtree = self.get_subtree_id(mother_id, p_id);
        if self.nodes[mother_id].kids[subtree].is_some() {
            return;
        }

        let last_node = self.nodes.len();
        let center = self.nodes[mother_id].center;
        let size = 0.5 * self.nodes[mother_id].size;
        let subtree_i8 = subtree as i8;
        self.nodes.push(Node {
            size,
            //compute the center
            //we multiply size by +1 or -1
            center: [
                center[0] + size * (2 * ((subtree_i8 & 4) >> 2) - 1) as f64,
                center[1] + size * (2 * ((subtree_i8 & 2) >> 1) - 1) as f64,
                center[2] + size * (2 * ((subtree_i8) & 1) - 1) as f64,
            ],
            center_of_mass: [0., 0., 0.],
            mass: 0.,
            particule: None,
            kids: [None; 8],
        });
        self.nodes[mother_id].kids[subtree] = Some(last_node as u32);
    }

    //recursively add a particule to a node
    fn add_particule_rec(&mut self, node_id: usize, particule_id: usize) {
        let particule = &mut self.particules[particule_id];
        self.nodes[node_id].mass += particule.mass;
        //println!("{}", particule_id);
        //println!("{}", particule.position[0]);

        if self.nodes[node_id].is_leaf() {
            if self.nodes[node_id].particule.is_some() {
                let id_p2 = self.nodes[node_id].particule.unwrap();
                self.nodes[node_id].particule = None;
                self.nodes[node_id].mass = 0.;
                self.make_kid(node_id, id_p2 as usize);
                self.make_kid(node_id, particule_id);
                self.add_particule_rec(node_id, id_p2 as usize);
                self.add_particule_rec(node_id, particule_id);
            } else {
                self.nodes[node_id].particule = Some(particule_id as u32);
            }
        } else {
            let subtree = self.get_subtree_id(node_id, particule_id);

            if self.nodes[node_id].kids[subtree].is_some() {
                self.add_particule_rec(
                    self.nodes[node_id].kids[subtree].unwrap() as usize,
                    particule_id,
                );
            } else {
                let last_node = self.nodes.len();
                self.make_kid(node_id, particule_id);
                self.add_particule_rec(last_node, particule_id);
            }
        }
    }

    //add a particule to the root of the tree
    //check if the particules is out of simulation then add it recursively to the tree
    //the particule is out of simulation of the distance to the center of density
    //is greater than the size of the root node
    //(so the simulation boundary is actually a sphere not a box!)
    fn add_particule(&mut self, particule_id: usize) {
        let p = &mut self.particules[particule_id];
        let d: f64 = p
            .position
            .iter()
            .zip(self.center.iter())
            .map(|(x, c)| (*x - *c) * (*x - *c))
            .sum::<f64>()
            .sqrt();
        if d > self.nodes[0].size {
            //println!("particule escaped simulation");

            //teleport the particule to the other side
            p.position
                .iter_mut()
                .zip(self.center.iter())
                .for_each(|(x, c)| *x = -0.95 * (*x) + 2. * c);
        }
        self.add_particule_rec(0, particule_id);
    }

    pub fn new(
        nb: usize,
        nb_save: usize,
        mu: f64,
        lambda: f64,
        virial: f64,
        theta: f64,
        is_plummer: bool,
        nb_bins: usize,
        nb_neighbors: usize,
        mu_init: f64,
        theta_init: f64,
    ) -> Tree {
        let mut tree = Tree {
            particules: generation(nb, is_plummer),
            nodes: Vec::new(),
            center: [0f64, 0f64, 0f64],
            rayons: [0f64, 0f64, 0f64],
            inertia_matrix: [0f64; 9],
            energy: 0f64,
            virial: 0f64,
            dynamical_time: 0f64,
            theta,
            dt: 0.01f64,
            mu,
            epsilon: 0.01f64,
            lambda,
            nb_save,
            nb_bins,
            nb_neighbors,
            mu_init,
            theta_init,
        };
        //root node
        tree.nodes.push(Node {
            //fixed root node size ...
            size: 40.,
            center: [0., 0., 0.],
            center_of_mass: [0., 0., 0.],
            mass: 0.,
            particule: None,
            kids: [None; 8],
        });

        for p_id in 0..tree.particules.len() {
            tree.add_particule(p_id);
        }
        tree.compute_center_of_mass(0);
        tree.compute_center();
        tree.compute_rayons();
        tree.compute_acceleration();
        tree.compute_energy();
        tree.compute_epsilon();
        tree.compute_dt();

        let virial_temp = tree.virial;
        //println!("energy{}", tree.energy);
        //println!("virial temp {}", virial_temp);

        //change the virial ratio
        tree.particules.par_iter_mut().for_each(|p| {
            p.speed
                .iter_mut()
                .for_each(|s| *s *= (virial / virial_temp).sqrt());
        });
        tree.rebuild_tree();
        tree.compute_center();
        tree.compute_rayons();
        tree.compute_acceleration();
        tree.compute_energy();
        tree.compute_epsilon();
        tree.compute_dt();
        tree
    }

    //rebuild the tree after the particules moved
    fn rebuild_tree(&mut self) {
        self.nodes.clear();
        self.nodes.push(Node {
            size: 40.,
            center: self.center,
            center_of_mass: [0., 0., 0.],
            mass: 0.,
            particule: None,
            kids: [None; 8],
        });

        for p_id in 0..self.particules.len() {
            self.add_particule(p_id);
        }
        self.compute_center_of_mass(0);
    }

    //recursively change the center of mass of the nodes
    fn compute_center_of_mass(&mut self, id: usize) {
        let kids = self.nodes[id].kids;
        for kid in kids.iter() {
            match kid {
                None => {
                    if self.nodes[id].particule.is_some() {
                        self.nodes[id].center_of_mass =
                            self.particules[self.nodes[id].particule.unwrap() as usize].position;
                    }
                }
                Some(kid_id) => {
                    self.compute_center_of_mass(*kid_id as usize);
                    for i in 0..3 {
                        self.nodes[id].center_of_mass[i] += self.nodes[*kid_id as usize]
                            .center_of_mass[i]
                            * self.nodes[*kid_id as usize].mass
                            / self.nodes[id].mass;
                    }
                }
            }
        }
    }

    //Compute the acceleration on p_id by walking the tree recursively,
    //and using the parameter theta to approximate long range interaction
    //the acceleration and potential is incremented in the array ap
    fn compute_acceleration_rec(&self, p_id: usize, node_id: usize) -> [f64; 4] {
        let p = &self.particules[p_id];
        let n = &self.nodes[node_id];
        //acceleration : (ap[0],ap[1],ap[2])
        //potential : ap[3]
        let mut ap = [0f64; 4];

        if n.particule.is_some() && n.particule.unwrap() == p_id as u32 {
            return ap;
        }

        let d_ = p
            .position
            .iter()
            .zip(n.center.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt();
        let d = f64::max(d_, self.epsilon);

        if f64::sqrt(3f64) * n.size / d < self.theta || n.particule.is_some() {
            let d_ = p
                .position
                .iter()
                .zip(n.center_of_mass.iter())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f64>()
                .sqrt();
            let d = f64::max(d_, self.epsilon);

            for i in 0..3 {
                ap[i] += n.mass / (d * d * d) * (n.center_of_mass[i] - p.position[i]);
            }
            ap[3] -= n.mass / d;
        } else {
            let kids = n.kids;
            for kid in kids.iter() {
                if kid.is_some() {
                    let ap_ = self.compute_acceleration_rec(p_id, kid.unwrap() as usize);
                    ap[0] += ap_[0];
                    ap[1] += ap_[1];
                    ap[2] += ap_[2];
                    ap[3] += ap_[3];
                }
            }
        }
        return ap;
    }

    //update the acceleration and potential of all particules
    fn compute_acceleration(&mut self) {
        //vec of ([acceleration, potential])
        let mut aps = vec![[0f64; 4]; self.particules.len()];
        aps.par_iter_mut().enumerate().for_each(|(p_id, ap)| {
            *ap = self.compute_acceleration_rec(p_id, 0);
        });
        self.particules
            .par_iter_mut()
            .zip(aps.par_iter())
            .for_each(|(p, ap)| {
                p.acceleration[0] = ap[0];
                p.acceleration[1] = ap[1];
                p.acceleration[2] = ap[2];
                p.potential = ap[3];
            });
    }

    pub fn leap_frog(&mut self) {
        let dt = self.dt;

        self.particules.par_iter_mut().for_each(|p| {
            p.speed[0] += 0.5 * dt * p.acceleration[0];
            p.speed[1] += 0.5 * dt * p.acceleration[1];
            p.speed[2] += 0.5 * dt * p.acceleration[2];

            p.position[0] += dt * p.speed[0];
            p.position[1] += dt * p.speed[1];
            p.position[2] += dt * p.speed[2];
        });

        self.rebuild_tree();
        self.compute_acceleration();

        self.particules.par_iter_mut().for_each(|p| {
            p.speed[0] += 0.5 * dt * p.acceleration[0];
            p.speed[1] += 0.5 * dt * p.acceleration[1];
            p.speed[2] += 0.5 * dt * p.acceleration[2];
        });
    }

    //Compute [R10, R50, R90]
    pub fn compute_rayons(&mut self) {
        let mut distances: Vec<f64> = self
            .particules
            .par_iter()
            .map(|p| {
                p.position
                    .iter()
                    .zip(self.center.iter())
                    .map(|(p_i, c_i)| (p_i - c_i) * (p_i - c_i))
                    .sum::<f64>()
                    .sqrt()
            })
            .collect();
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.rayons = [
            distances[distances.len() / 10],
            distances[distances.len() / 2],
            distances[distances.len() - distances.len() / 10],
        ]
    }

    //update epsilon
    //epsilon = (4/(3*N*pi))^(1/3) * R50  / lambda
    pub fn compute_epsilon(&mut self) {
        self.epsilon = (4f64 / (3f64 * self.particules.len() as f64 * std::f64::consts::PI))
            .powf(1f64 / 3f64)
            * self.rayons[1]
            / self.lambda;
    }

    //compute total energy and virial and update cinetic energy of each particules
    pub fn compute_energy(&mut self) {
        self.particules.par_iter_mut().for_each(|p| {
            let s_sq = p.speed.iter().map(|s| s * s).sum::<f64>();
            p.cinetic = 0.5 * p.mass * s_sq;
        });

        let e_c: f64 = self.particules.par_iter().map(|p| p.cinetic).sum();
        let e_p: f64 = self
            .particules
            .par_iter()
            .map(|p| p.mass * 0.5 * p.potential)
            .sum();

        self.energy = e_c + e_p;
        self.virial = 2. * e_c / e_p;
    }

    //update dt and dynamical time
    pub fn compute_dt(&mut self) {
        let ro = self
            .particules
            .par_iter()
            .map(|p| {
                p.position
                    .iter()
                    .zip(self.center.iter())
                    .map(|(pos_i, c_i)| (*pos_i - *c_i) * (*pos_i - *c_i))
                    .sum::<f64>()
                    .sqrt()
            })
            .sum::<f64>()
            / self.particules.len() as f64;
        let s = self
            .particules
            .par_iter()
            .map(|p| p.speed.iter().map(|s_i| s_i * s_i).sum::<f64>().sqrt())
            .sum::<f64>()
            / self.particules.len() as f64;
        self.dynamical_time = ro / s;
        self.dt = self.dynamical_time / self.mu;
    }

    //used to find the nearest neighbor
    fn sphere_touch_node(node: &Node, particule: &Particule, r: f64) -> bool {
        let mut dmin = 0f64;
        let boxmin = [
            node.center[0] - node.size,
            node.center[1] - node.size,
            node.center[2] - node.size,
        ];
        let boxmax = [
            node.center[0] + node.size,
            node.center[1] + node.size,
            node.center[2] + node.size,
        ];
        for i in 0..3 {
            if particule.position[i] < boxmin[i] {
                dmin += (particule.position[i] - boxmin[i]) * (particule.position[i] - boxmin[i]);
            } else if particule.position[i] > boxmax[i] {
                dmin += (particule.position[i] - boxmax[i]) * (particule.position[i] - boxmax[i]);
            }
        }
        dmin <= r
    }

    //compute the local density of the particule p_id
    //for the calculation of the center of density
    //it use the octree to reduce the complexity of finding the k-nearest-neighbor
    fn compute_local_density(&self, p_id: usize, node_id: usize, radii: &mut Vec<f64>) {
        let p = &self.particules[p_id];
        let n = &self.nodes[node_id];

        //if every points in this subspace is to far from the particule we don't need to explore it
        if !Tree::sphere_touch_node(n, p, *radii.last().unwrap()) {
            return;
        }

        if n.particule.is_some() {
            //if this node is a leaf, we check if the particule is a k-NN
            let p_2 = &self.particules[n.particule.unwrap() as usize];
            let d = p
                .position
                .iter()
                .zip(p_2.position.iter())
                .map(|(p1, p2)| (p1 - p2) * (p1 - p2))
                .sum();
            if d < *radii.last().unwrap() {
                radii.push(d);
                radii.sort_by(|a, b| a.partial_cmp(b).unwrap());
                radii.pop();
            }
        } else {
            //else we first explore the branch of the tree that contains the particule
            let subtree = self.get_subtree_id(node_id, p_id);
            if n.kids[subtree].is_some() {
                self.compute_local_density(p_id, n.kids[subtree].unwrap() as usize, radii);
            }

            //then we explore the others branches
            for (_, kid) in n
                .kids
                .iter()
                .enumerate()
                .filter(|(i, k)| *i != subtree && k.is_some())
            {
                self.compute_local_density(p_id, kid.unwrap() as usize, radii);
            }
        }
    }

    //Compute the center of density
    pub fn compute_center(&mut self) {
        let k = self.nb_neighbors;
        let mut center = [0f64; 3];
        let mut densites = vec![0f64; self.particules.len()];

        (0..self.particules.len())
            .into_par_iter()
            .zip(densites.par_iter_mut())
            .for_each(|(p_id, d)| {
                let mut radii = vec![std::f64::INFINITY; k];
                self.compute_local_density(p_id, 0, &mut radii);
                let r_sq = radii.last().unwrap();
                *d = 1. / (r_sq * r_sq.sqrt());
            });

        center[0] = self
            .particules
            .par_iter()
            .zip(densites.par_iter())
            .map(|(p, d)| p.position[0] * d)
            .sum();
        center[1] = self
            .particules
            .par_iter()
            .zip(densites.par_iter())
            .map(|(p, d)| p.position[1] * d)
            .sum();
        center[2] = self
            .particules
            .par_iter()
            .zip(densites.par_iter())
            .map(|(p, d)| p.position[2] * d)
            .sum();

        let sum: f64 = densites.par_iter().sum();
        center[0] /= sum;
        center[1] /= sum;
        center[2] /= sum;

        self.center = center;
    }

    pub fn compute_inertia_matrix(&mut self) {
        let a: f64 = self
            .particules
            .par_iter()
            .map(|p| {
                (p.position[1] - self.center[1]).powf(2f64)
                    + (p.position[2] - self.center[2]).powf(2f64)
            })
            .sum();
        let b: f64 = self
            .particules
            .par_iter()
            .map(|p| {
                (p.position[0] - self.center[0]).powf(2f64)
                    + (p.position[2] - self.center[2]).powf(2f64)
            })
            .sum();
        let c: f64 = self
            .particules
            .par_iter()
            .map(|p| {
                (p.position[1] - self.center[1]).powf(2f64)
                    + (p.position[0] - self.center[0]).powf(2f64)
            })
            .sum();
        let d: f64 = self
            .particules
            .par_iter()
            .map(|p| (p.position[0] - self.center[0]) * (p.position[1] - self.center[1]))
            .sum();
        let e: f64 = self
            .particules
            .par_iter()
            .map(|p| (p.position[0] - self.center[0]) * (p.position[2] - self.center[2]))
            .sum();
        let f: f64 = self
            .particules
            .par_iter()
            .map(|p| (p.position[1] - self.center[1]) * (p.position[2] - self.center[2]))
            .sum();

        self.inertia_matrix[0] = a;
        self.inertia_matrix[1] = -d;
        self.inertia_matrix[2] = -e;

        self.inertia_matrix[3] = -d;
        self.inertia_matrix[4] = b;
        self.inertia_matrix[5] = -f;

        self.inertia_matrix[6] = -e;
        self.inertia_matrix[7] = -f;
        self.inertia_matrix[8] = c;
    }
}
