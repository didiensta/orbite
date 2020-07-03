"""
Compacte les résultats d'une simulation en un unique fichier.
 - all_data.pickle (pour Python avec le module 'pickle')
 - all_data.mat (pour MATLAB / Python avec le module 'scipy')

Les données sont stockées sous les noms et formats suivants :
    't': Nt array,
    'c': Nt array,
    'positions': Nt x Np x 3 array,
    'speeds': Nt x Np x 3 array,
    'energy': Nt x Np x 2 array (cinétique, potentielle),
    'radii': Nt x Nbins array,
    'density': Nt x Nbins array,
    'rayons': Nt x 3 array (R10, R50, R90),
    'inertia_matrix': Nt x 9 array,
    'total_energy': Nt array,
    'virial': Nt array,
    'dynamical_time': Nt array,
    'espilon': Nt array

Fonctionne pour les différentes versions du code de simulation.

Utilisation (en ligne de commande) : 
$ python3 convert.py chemin_vers_le_dossier_de_la_simulation/
"""

import sys
import pickle
import csv
from os import path, listdir

try:
    import cbor2
except ImportError as err:
    print("Warning:", err)
    print("Le script va planter si il tente de lire un fichier CBOR.")

try:
    import numpy as np
    from scipy import io as sio
except ImportError as err:
    print("Error:", err)
    sys.exit("Le script nécessite les modules 'numpy' et 'scipy'")


def load_data(folder, ext, load_function):
    file_count = 0
    while path.exists(path.join(folder, "data_{}.{}".format(file_count, ext))):
        file_count += 1
    
    with open(path.join(folder, "data_0." + ext), 'rb') as data_file:
        data = load_function(data_file)

    N = len(data['positions'])
    Nbins = len(data['radii'])
    all_data = {
        't': np.empty(file_count),
        'c': np.empty(file_count),
        'positions': np.empty((file_count, N, 3)),
        'speeds': np.empty((file_count, N, 3)),
        'energy': np.empty((file_count, N, 2)),
        'radii': np.empty((file_count, Nbins)),
        'density': np.empty((file_count, Nbins)),
        'rayons': np.empty((file_count, 3)),
        'inertia_matrix': np.empty((file_count, 9)),
        'total_energy': np.empty(file_count),
        'virial': np.empty(file_count),
        'dynamical_time': np.empty(file_count),
        'espilon': np.empty(file_count)
    }

    print("Lecture de {} fichiers... ".format(file_count), end='')
    for k in range(file_count):
        with open(path.join(folder, "data_{}.{}".format(k, ext)), 'rb') as data_file:
            data = load_function(data_file)                
            for key in ['t', 'c', 'virial', 'dynamical_time', 'espilon']:
                all_data[key][k] = data[key]
            for key in ['positions', 'speeds']:
                all_data[key][k, :, :] = data[key]
            for key in ['radii', 'density', 'rayons', 'inertia_matrix']:
                all_data[key][k, :] = data[key]
            if 'total_energy' in data:
                all_data['energy'][k, :, :] = data['energy']
                all_data['total_energy'][k] = data['total_energy']
            else:
                all_data['total_energy'][k] = data['energy']
    print("Fait.")
    return all_data


def load_data_old_format(folder):
    all_data = {
        't': [],
        'positions': [],
        'radii': [],
        'density': [],
        'rayons': [],
        'inertia_matrix': [],
        'energy': [],
        'total_energy': [],
        'virial': [],
        'dynamical_time': [],
    }
    with open(path.join(folder, "infos.csv"), 'r') as data_file:
        reader = csv.reader(data_file, delimiter=';')
        for row in reader:
            all_data['t'].append(float(row[0]))
            all_data['dynamical_time'].append(float(row[1]))
            all_data['total_energy'].append(float(row[2]))
            all_data['virial'].append(float(row[3]))
            all_data['rayons'].append([float(r) for r in row[4:7]])

    with open(path.join(folder, "inertia_matrix.csv"), 'r') as data_file:
        reader = csv.reader(data_file, delimiter=';')
        for row in reader:
            all_data['inertia_matrix'].append([float(c) for c in row[:9]])

    for k in range(len(all_data['t'])):
        positions = []
        energy = []
        with open(path.join(folder, "positions/{}.csv".format(k)), 'r') as data_file:
            reader = csv.reader(data_file, delimiter=';')
            for row in reader:
                positions.append([float(c) for c in row[:3]])
                energy.append([float(c) for c in row[3:5]])
        all_data['positions'].append(positions)
        all_data['energy'].append(energy)

    for filename in listdir(path.join(folder, "densities")):
        radii = []
        density = []
        with open(path.join(folder, "densities", filename), 'r') as data_file:
            reader = csv.reader(data_file, delimiter=';')
            for row in reader:
                radii.append(float(row[0]))
                density.append(float(row[1]))
        all_data['radii'].append(radii)
        all_data['density'].append(density)

    for key in all_data:
        all_data[key] = np.array(all_data[key])

    return all_data
            

def write_data(filepath, data):
    ext = filepath.split('.')[-1]
    if ext == "mat":
        sio.savemat(filepath, data)
    elif ext == "pickle":
        with open(filepath, 'wb') as data_file:
            pickle.dump(data, data_file)
    else:
        print("Extention de ficher non reconnue.")

        
if __name__ == "__main__":

    if len(sys.argv) < 2:
        sys.exit("Entrer le chemin vers le dossier de la simulation")
    
    folder = sys.argv[1]
    if not path.exists(folder):
        sys.exit("Dossier de simulation introuvable")

    if path.exists(path.join(folder, "data_0.cbor")):
        all_data = load_data(folder, "cbor", cbor2.load)
    elif path.exists(path.join(folder, "data_0.pickle")):
        all_data = load_data(folder, "pickle", pickle.load)
    elif path.exists(path.join(folder, "infos.csv")):
        all_data = load_data_old_format(folder)
    else:
        sys.exit("Fichiers introuvables.")

    print("Ecriture de {}/all_data.pickle... ".format(path.dirname(folder)), end='')
    write_data(path.join(folder, "all_data.pickle"), all_data)
    print("Fait.")

    print("Ecriture de {}/all_data.mat... ".format(path.dirname(folder)), end='')
    write_data(path.join(folder, "all_data.mat"), all_data)
    print("Fait.")
