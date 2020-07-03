#!/bin/python3
"""
Utilisable en ligne de commande :
$ python3 verify.py all_data.pickle
"""

import sys
import pickle
import numpy as np
from os import path
from matplotlib import pyplot as plt


def verifications(sim_data, dirname='.', N_dens=5):
    """
    Trace et enregistre les graph de la simulation de données 'sim_data'.
    Trace 'N_dens' courbes pour la densité.
    """

    t = sim_data['t']
    Nt = len(t)
    energy = sim_data['total_energy']
    index = np.linspace(0, Nt-1, N_dens, dtype=int)
    r = np.array([sim_data['radii'][i, :-1] / sim_data['rayons'][i, -1] for i in index])
    rho = sim_data['density'][index, :-1]

    var_energy = (energy - np.mean(energy)) / np.mean(energy)
    inertia_matrices = np.reshape(sim_data['inertia_matrix'], (Nt, 3, 3))
    eigs = np.linalg.eigvalsh(inertia_matrices)
    a1 = eigs[:, 0] / eigs[:, 1]
    a2 = eigs[:, 2] / eigs[:, 1]

    fig = plt.figure(figsize=(7.2, 8.2))
    axs = fig.subplots(4, sharex=True, gridspec_kw={'hspace': 0, 'right': 0.8})

    axs[0].plot(t, var_energy*100, label=r"$\frac{E - \bar{E}}{\bar{E}}\,(\%)$")

    axs[1].plot(t, sim_data['virial'], label="virial")

    axs[2].plot(t, a1, label=r"$a_1$")
    axs[2].plot(t, a2, label=r"$a_2$")

    axs[3].plot(t, sim_data['rayons'][:, 2], label=r"$R_{90}$")
    axs[3].plot(t, sim_data['rayons'][:, 1], label=r"$R_{50}$")
    axs[3].plot(t, sim_data['rayons'][:, 0], label=r"$R_{10}$")
    axs[3].set_xlabel(r"$t\,/\,T_d$", fontsize=11)

    for a in axs:
        a.tick_params(labelsize=11)
        a.secondary_xaxis('top').tick_params(top=True, labeltop=False, direction='in')
        a.secondary_xaxis('bottom').tick_params(bottom=True, labelbottom=False, direction='in')
        a.legend(bbox_to_anchor=(1.05, 1), loc='upper left', borderaxespad=0., fontsize=11)

    fig.savefig(path.join(dirname, "verification.png"))

    fig = plt.figure()
    for i in range(N_dens):
        plt.loglog(r[i, :], rho[i, :], label=r"$t\,/\,T_d = {:.3f}$".format(t[index[i]]))
    plt.tick_params(labelsize=11)
    plt.xlabel(r"$r\,/\,R_{50}$", fontsize=11)
    plt.ylabel(r"$\rho$", fontsize=11)
    plt.legend(fontsize=11)

    fig.savefig(path.join(dirname, "density.png"))
    # plt.show()
    plt.close()

if __name__ == "__main__":

    if len(sys.argv) < 2:
        sys.exit("Entrer le chemin vers le fichier de la simulation")
    filepath = sys.argv[1]
    if not path.isfile(filepath):
        sys.exit("Fichier de données introuvable")
    
    with open(filepath, 'rb') as data_file:
        sim_data = pickle.load(data_file)
    verifications(sim_data, path.dirname(filepath))