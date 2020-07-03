#!/bin/python3
"""
pas fini
Utilisable en ligne de commande :
$ python3 trajectoire.py all_data.pickle
"""


import sys
import pickle
import numpy as np
from os import path
from matplotlib import pyplot as plt
from scipy.signal import find_peaks

from isochrony import *

def traj_tau_E(t, x, y, dy, pente, c, point_count):
    nb = len(x)
    i = 0
    index = np.logical_not(np.isnan(x[i, :]))
    while np.sum(index) <= point_count and i < nb-1:
        i += 1
        index = np.logical_not(np.isnan(x[i, :]))

    all_part = np.reshape(np.argwhere(index), np.sum(index))
    s = np.argsort(y[i, index])
    all_part = all_part[s]
    part = all_part[np.linspace(0, len(all_part)-1, point_count, dtype=int)]

    for p in part:
        plt.plot(x[:, p], y[:, p], '-') # , label=r"\sigma_E = {:.3f}".format(dy[p]))
        i = 0
        while np.isnan(x[i, p]):
            i += 1
        plt.plot(x[i, p], y[i, p], 'or')
        for k in range(i, int(len(t)/3), 30):
            plt.plot(x[k, p], y[k, p], '+k')
            plt.text(x[k, p]+0.01, y[k, p]+0.01, f"{t[k]:.2f}", fontsize=8)

    
    # x1 = np.min(x[np.logical_not(np.isnan(x))])
    # x2 = np.max(x[np.logical_not(np.isnan(x))])
    # for i in np.linspace(0, len(pente)-1, 8, dtype=int):
    #     plt.plot([x1, x2], [pente[i]*x1 + c[i], pente[i]*x2 + c[i]], ':k', label="$t/Td = {:.2f}$".format(t[i]))
    #     plt.text(x2 + 0.01, pente[i]*x2 + c[i] + 0.01, r"$t/Td={:.2f}, p={:.2f}$".format(t[i], pente[i]), fontsize=8)
    # plt.xlim(x1-0.1, x2+0.75)
    plt.xlabel(r"$\ln(\tau)$", fontsize=12)
    plt.ylabel(r"$\ln(-E)$", fontsize=12)
    # plt.legend()

    plt.savefig(path.join(path.dirname(filepath), "traj_1.png"))
    plt.show()
    plt.close()

    plan = is_planar(sim_data)
    t = sim_data['t']
    pos = sim_data['positions'][:, plan, :]
    R = np.sqrt(np.sum(pos**2, 2))
    print(t.shape, pos.shape, R.shape)

    for p in part:
        plt.plot(t, R[:, p], label=str(p))
    plt.xlabel(r"t")
    plt.ylabel(r"R")
    plt.legend()

    plt.savefig(path.join(path.dirname(filepath), "traj_2.png"))
    plt.close()

if __name__ == "__main__":

    if len(sys.argv) < 2:
        sys.exit("Entrer le chemin vers le fichier de la simulation")
    filepath = sys.argv[1]
    if not path.isfile(filepath):
        sys.exit("Fichier de données introuvable")
    
    with open(filepath, 'rb') as data_file:
        sim_data = pickle.load(data_file)
    x, y, dy, pente, c = pente_isochrony(sim_data, path.dirname(filepath))

    traj_tau_E(sim_data['t'], x, y, dy, pente, c, 12)

