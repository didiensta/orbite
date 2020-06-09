# Orbite

## Install and run

Install Rust: https://www.rust-lang.org/tools/install

Use Cargo to build and run.

Build:

	cargo build --release

Run:

	./target/release/orbite configuration_file.ini

Or build and run at the same time:

	cargo run --release configuration_file.ini

## Configuration file

Use a configuration file to specify all the parameters of the simulation.
See conf.ini for an example.

## Changements

### Liste des modifications

- Fonction `from_csv_gen` ajoutée dans `particules.rs`.
- Prise en compte d'un argument supplémentaire `from_csv` dans le fichier de configuration.
- Adaptation de `generation` dans `particules.rs` et de l'objet `Tree` pour l'argument `from_csv`.

### Utilisation

Run :
```
./target/release/orbite configuration_file.ini < initial_condition_file.csv
```

Structure du fichier .csv contenat les conditions initiales (une ligne par particule) :
```
...
position_x;position_y;position_z;vitesse_x;vitesse_y;vitesse_z
...
```

