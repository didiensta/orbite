#!/bin/python3
"""
Utilisable en ligne de commande pour le tracé de pente en fonction du temps :
$ python3 verify.py all_data.pickle

ou $ python3 verify.py all_data.pickle nb
avec nb un entier représentant le nombre minimal de point de données
pour faire un fit et calculer une période.

"""


import sys
import pickle
import numpy as np
from os import path
from time import time
from matplotlib import pyplot as plt
from scipy.signal import find_peaks


def cut_sections(x, y, seuil=0.01):
    """
    Detecte la position des pics dans le signal (x, y) pour le
    découper en sections.
    """

    min_peak_prominence = seuil * (np.max(y) - np.min(y))   
    peaks, _ = find_peaks(y, prominence=min_peak_prominence) # Positions (en indices) des pics 
    sections_err = np.empty(len(peaks)) # Erreur sur la position de début de la section

    count = 0
    for i in peaks:
        # Interpolation pour corriger en partie l'erreur due à la discrétisation.
        p = np.polyfit(x[i-1:i+2], y[i-1:i+2], 2)
        x_correct = -p[1]/(2*p[0])
        y_correct = np.polyval(p, x_correct)
        delta = np.sqrt((x[i] - x_correct)**2 + (y[i] - y_correct)**2) # Erreur (en distance)
        # Signe de l'erreur
        if x_correct <= x[i]:
            sections_err[count] = delta
        else:
            sections_err[count] = -delta
        count += 1

    return peaks, sections_err


def apocenter(y, seuil=0.01):
    """
    Retourne l'apocentre moyen du signal 'y'.
    Retourne 'None' si aucun extremum local n'est trouvé.
    """
    min_peak_prominence = seuil * (np.max(y) - np.min(y))
    peaks, _ = find_peaks(y, prominence=min_peak_prominence)
    if peaks.size:
        return np.mean(y[peaks])


def period(x, y, energy):
    """
    Calcule une valeur approchée de la "periode locale" à chaque instant pour le signal (x, y)
    ainsi q'une valeur d'énergie associée calculé selon 'energy'.

    x, y et energy sont 3 vecteurs de même taille

    Retourne 2 vecteur (périodes et energies) de même dimension que x, y, et energy,
    comportant la valeur Nan (numpy.NaN) aux indices où le calcule de période est impossbles
    """

    sec, sec_err = cut_sections(x, y)

    dx2 = np.diff(x)**2
    dy2 = np.diff(y)**2

    tau = np.empty(len(x)) # Periodes
    tau[:] = np.NaN # Initialement rempli de NaN

    E = np.empty(len(x)) # Energies
    E[:] = np.NaN # Initialement rempli de NaN

    # Si le nombre de pics détectés par 'cut_sections' est inssufisant
    if len(sec) <= 4:
        return tau, E

    for i in range(len(sec)-2):
        k_start = 0
        l1 = np.sum(np.sqrt(dx2[sec[i]:sec[i+1]-1] + dy2[sec[i]:sec[i+1]-1])) + sec_err[i] - sec_err[i+1]
        l2 = np.sum(np.sqrt(dx2[sec[i+1]:sec[i+2]-1] + dy2[sec[i+1]:sec[i+2]-1])) + sec_err[i+1] - sec_err[i+2]

        D1 = sec_err[i]
        for j in range(sec[i+1] - sec[i]):
            D1 +=  np.sqrt(dx2[sec[i]+j] + dy2[sec[i]+j])
            d1 = D1/l1
            D2 = np.sum(np.sqrt(dx2[sec[i+1]:sec[i+1]+k_start] + dy2[sec[i+1]:sec[i+1]+k_start])) + sec_err[i+1]

            for k in range(k_start, sec[i+2] - sec[i+1] +1):
                D2_ans = D2
                D2 += np.sqrt(dx2[sec[i+1]+k] + dy2[sec[i+1]+k])
                d2 = D2/l2

                if d2 >= d1:
                    m_d = (x[sec[i+1]+k] - x[sec[i+1]+k-1]) / (d2 - D2_ans/l2)
                    x_next = m_d * (d1 - d2) + x[sec[i+1]+k]

                    i_middle = int((sec[i+1]+k + sec[i]+j) / 2)
                    x_middle = (x_next + x[sec[i]+j]) / 2
                    
                    m_e = (energy[i_middle+1] - energy[i_middle]) / (x[i_middle+1] - x[i_middle])
                    E_middle = m_e * (x_middle - x[i_middle]) + energy[i_middle]

                    tau[sec[i]+j] = x_next # - x[sec[i]+j]
                    E[sec[i]+j] = E_middle

                    k_start = k
                    break

    return tau-x, E

def weighted_lin_fit(x, y, dy):
    """
    fit linéaire avec poids pour y±dy = s*x + c
    avec 'dc' et 'ds' les incertitudes sur 'c' et 's'
    """
    w = 1 / (dy**2)
    Delta = np.sum(w) * np.sum(w * x**2) - np.sum(w * x)**2
    c = 1/Delta * (np.sum(w * x**2) * np.sum(w * y) - np.sum(w * x) * np.sum(w * x * y))
    s = 1/Delta * (np.sum(w) * np.sum(w * x * y) - np.sum(w * x) * np.sum(w * y))
    dc = np.sqrt(np.sum(w * x**2) / Delta)
    ds = np.sqrt(np.sum(w) / Delta)
    return c, s, dc, ds


def is_planar(sim_data, lim=0.2):
    """
    'sim_data' les données compactées par convert.py d'une simulation.
    Retourne une liste de booléen de taille nombre_de_particules indiquant
    si l'orbite de chacune des particules peut être considérée comme plane

    Calqué sur l'article de Jérôme Perez (section 3.2 page 14)
    """
    
    Nt, Np, _ = sim_data['positions'].shape

    if 'speeds' in sim_data.keys():
        L = np.cross(sim_data['positions'], sim_data['speeds'])
    else:
        V = np.empty((Nt-2, Np, 3))
        for i in range(1, Nt-1):
            V[i-1, :, :] = 0.5 * (sim_data['positions'][i+1, :, :] - sim_data['positions'][i-1, :, :]) / (sim_data['t'][i] - sim_data['t'][i-1])
        L = np.cross(sim_data['positions'][1:Nt-1, :, :], V)
    L = np.mean(L, 0)

    d = np.sum(L * sim_data['positions'], 2) / np.sqrt(np.sum(L**2, 1)) # Ecarts au plan
    delta = np.max(d, 0) - np.min(d, 0)

    planar = []
    R = np.sqrt(np.sum(sim_data['positions']**2, 2))
    for p in range(Np):
        ra = apocenter(R[:, p])
        if ra is not None:
            planar.append(delta[p] >= lim*ra)
        else:
            planar.append(False)

    return planar
        

def pente_isochrony(sim_data, dirname, min_point_count=100, plotting=False):
    """
    Calcule la pente du graph (ln(tau), ln(-E)) à tout instant où cela est possible
    pour tracer la pente en fonction du temps.
    Trace également le graph de la dernière pente calculée (ce qui n'est pas vraiment judicieux
    car la dernière pente calculée est techniquement la moins "précise" de toutes)

    Renvoie :
        'x' ln(tau): Nt x Np array, rempli de np.NaN quand le calcul est impossible,
        'y' ln(-E): Nt x Np array, rempli de np.NaN aux mêmes indices que 'x',
        'dy' variance(ln(-E)) : Np array,
        'pente' la pente: Nt array, rempli de np.NaN quand le fit est impossible,
        'coef' l'ordonée à l'origine du fit: Nt array, rempli de np.NaN quand le fit est impossible,
    """

    print("Exclusion des orbites non planes...")
    plan = is_planar(sim_data)

    t = sim_data['t']
    pos = sim_data['positions'][:, plan, :]
    R = np.sqrt(np.sum(pos**2, 2))
    energy = sim_data['energy'][:, plan, :]
    energy = np.sum(energy, 2)
    
    Nt, Np, _ = sim_data['positions'].shape

    print("Calcul des périodes...")
    t1 = time()
    x = np.empty((Nt, Np))
    y = np.empty((Nt, Np))
    dy = np.empty(Np)
    for p in range(Np):
        tmp = period(t, R[:, p], energy[:, p])
        tau, E = tmp[0], tmp[1]
        log_E = np.log(-energy[:, p])
        x[:, p] = np.log(tau)
        y[:, p] = np.log(-E)
        dy[p] = np.sqrt(np.sum((log_E - np.mean(log_E))**2) / len(log_E))

    print(time()-t1, "s")

    print("Calcul des pentes...")
    pente = np.empty(Nt)
    pente[:] = np.NaN
    coef = np.empty(Nt)
    coef[:] = np.NaN
    d_pente = np.empty(Nt)
    d_pente[:] = np.NaN
    err = np.empty(Nt)
    err[:] = np.NaN

    index_last = []
    last = 0
    for i in range(Nt):
        index = np.logical_not(np.isnan(x[i, :]))
        if np.sum(index) >= min_point_count:
            last = i
            index_last = index
            c, s, _, ds = weighted_lin_fit(x[i, index], y[i, index], dy[index])
            pente[i] = s
            d_pente[i] = ds
            coef[i] = c
            err[i] = np.mean((y[i, index] - np.polyval((s, c), x[i, index]))**2)

    if plotting:
        print("Création des figures...")
        fig = plt.figure(figsize=(6.8, 5.2))
        ax1 = plt.subplot(111)
        ax1.tick_params(labelsize=11)
        ax1.plot(t, pente, '-r')
        ax1.plot(t, pente + d_pente, '--r')
        ax1.plot(t, pente - d_pente, '--r')
        ax1.plot([t[0], t[-1]], [-2/3, -2/3], ':k')
        ax1.set_xlabel(r"$t\,/\,T_d$")
        ax1.set_ylabel("pente", fontsize=11)

        plt.savefig(path.join(dirname, "pente_nb_{}.png".format(min_point_count)))
        # plt.show()
        plt.close()

        x_fit = np.linspace(np.min(x[last, index_last]), np.max(x[last, index_last]))
        y_fit = np.polyval([s, c], x_fit)

        plt.errorbar(x[last, index_last], y[last, index_last], yerr=dy[index_last], fmt='+')
        plt.plot(x_fit, y_fit, '-k', linewidth=2)
        plt.xlabel(r"$\ln(\tau)$")
        plt.ylabel(r"$\ln(-E)$")
        plt.title(r"Pente : ${:.3f}\pm{:.3f}$".format(s, ds))
        plt.savefig(path.join(dirname, "last_pente_nb_{}.png".format(min_point_count)))
        plt.close()

    return x, y, dy, pente, coef


if __name__ == "__main__":

    if len(sys.argv) < 2:
        sys.exit("Entrer le chemin vers le fichier de la simulation")
    filepath = sys.argv[1]
    if not path.isfile(filepath):
        sys.exit("Fichier de données introuvable")
    
    if len(sys.argv) > 2:
        nb = int(sys.argv[2])
    else:
        nb = 100

    with open(filepath, 'rb') as data_file:
        sim_data = pickle.load(data_file)
    pente_isochrony(sim_data, path.dirname(filepath), nb, plotting=True)