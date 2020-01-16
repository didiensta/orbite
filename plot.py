"""
Plot l'evolution de l'energie, des rapports d'axes, du viriel, des rayons
et le profil de densite.
"""
import sys
import os
import glob
import csv
import numpy as np
import matplotlib
import matplotlib.pyplot as plt
matplotlib.use('Agg')
#ecrire_infos(tps, tpsdyns, energies, viriels, rayons_10, rayons_50, rayons_90, step);

DOSSIER = sys.argv[1]

def temps_dynamique():
    """ retourne une np array correspondant au temps a chaque pas """
    times = []
    with open(DOSSIER + "/infos.csv") as csvDataFile:
        csvReader = csv.reader(csvDataFile, delimiter=';')
        for row in csvReader:
            time = float(row[0])
            times.append(time)
    return np.array(times)

def plot_energie():
    """ plot la variation de l'erreur de l'energie en % :
        (e_0 - e_t)/ e_0 * 100"""
    energies = []
    with open(DOSSIER + "/infos.csv") as csvDataFile:
        csvReader = csv.reader(csvDataFile, delimiter=';')
        for row in csvReader:
            energie = float(row[2])
            energies .append(energie)
    temps = temps_dynamique()
    e0 = np.abs(energies[0])
    energies = (np.abs(e0)-np.abs(energies))/e0 * 100
    plt.xlabel(r"$T_d$")
    plt.ylabel("% d'erreur sur l'energie")
    plt.plot(temps, energies)
    plt.savefig(DOSSIER + "/energie_erreur.png")
    plt.close()

def plot_viriel():
    """ plot l'evolution du viriel """
    energies = []
    with open(DOSSIER + "/infos.csv") as csvDataFile:
        csvReader = csv.reader(csvDataFile, delimiter=';')
        for row in csvReader:
            energie = float(row[3])
            energies.append(energie)

    temps = temps_dynamique()
    plt.xlabel(r"$T_d$")
    plt.ylabel("viriel")
    plt.plot(temps, energies)
    plt.savefig(DOSSIER + "/viriels.png")
    plt.close()

#plot la variation des rayons
def plot_rayons():
    """ plot l'evolution des rayons contenant 10% 50% et 90% de la masse """
    rs_10 = []
    rs_50 = []
    rs_90 = []
    with open(DOSSIER + "/infos.csv") as csvDataFile:
        csvReader = csv.reader(csvDataFile, delimiter=';')
        for row in csvReader:
            r_10 = float(row[4])
            r_50 = float(row[5])
            r_90 = float(row[6])
            rs_10.append(r_10)
            rs_50.append(r_50)
            rs_90.append(r_90)

    temps = temps_dynamique()
    plt.plot(temps, rs_10)
    plt.plot(temps, rs_50)
    plt.plot(temps, rs_90)
    plt.xlabel(r"$T_d$")
    plt.ylabel("rayon")
    plt.legend(["R10", "R50", "R90"])
    plt.savefig(DOSSIER + "/rayons.png")
    plt.close()

def plot_axes():
    """ plot l'evolution des deux rapport d'axes """

    #tableau contenant a chaque pas de temps les 3 valeurs propres
    #de la matrice d'inertie
    tab_valeurs_propres = []
    with open(DOSSIER + "/inertia_matrix.csv") as csvDataFile:
        csvReader = csv.reader(csvDataFile, delimiter=';')
        for row in csvReader:
            #chaque ligne corespond a la matrice d'inertie
            m = np.zeros((3, 3))
            for i in range(0, len(row)-1):
                m[i%3][i//3] = float(row[i])
            #on calcul les valeurs propres de la matrice
            valeurs_propres, _ = np.linalg.eig(m)
            tab_valeurs_propres.append(valeurs_propres)

    temps = temps_dynamique()
    tab_valeurs_propres = np.sort(tab_valeurs_propres)
    a1 = []
    a2 = []
    for valeurs_propres in tab_valeurs_propres:
        a1.append(valeurs_propres[0]/valeurs_propres[1])
        a2.append(valeurs_propres[2]/valeurs_propres[1])
    fig = plt.figure()
    plt.plot(temps, a1)
    plt.plot(temps, a2)
    plt.legend([r"$a_1$", r"$a_2$"])
    plt.xlabel(r"$T_d$")
    plt.savefig(DOSSIER + "/axes.png")
    plt.close()

def plot_histo_densite_plusieurs():
    """ plot le profil de densite a 4 moments """
    #lit les fichiers du profil de densite,
    #on les tri pour les mettre dans le bon ordre
    listcsv = sorted(glob.glob(DOSSIER + "/densities/*.csv"), key=os.path.getmtime)
    #puis on en conserve que 4 
    listcsv = [listcsv[1], listcsv[len(listcsv)//4], listcsv[len(listcsv)//2],
            listcsv[-1]]

    #on plot la densite pour chacun des 4 temps choisit
    for file in listcsv:
        rs = []
        ds = []
        with open(file) as csvDataFile:
            csvReader = csv.reader(csvDataFile, delimiter=';')
            for row in csvReader:
                r = float(row[0])
                d = float(row[1])
                if( r != 0 and d != 0):
                    rs.append(r)
                    ds.append(d)
        if( len(rs) == 0):
            print("Erreur pour le calcul de la densite, \
                la densite est nulle partout (volume trop grand?)")

        #on normalise la densite, et on divise le rayon par R50
        rs = np.array(rs)/rs[len(rs)//2]
        ds = np.array(ds)/np.sum(ds)
        #on enleve la fin pour avoir un joli graphe
        rs = rs[0:-5]
        ds = ds[0:-5]

        #plot en echelle log
        plt.loglog(rs, ds)

    temps = temps_dynamique()
    temps = [temps[0], temps[len(temps)//4], temps[len(temps)//2],
            temps[len(temps)//4 *3]]

    for i in range(0,len(listcsv)):
        listcsv[i] = r"$T_d$ : " + str(int(round(temps[i])))

    plt.xlabel(r"$\ln(r/R_{50})$")
    plt.ylabel(r"$\ln(densite)$")
    plt.legend(listcsv)
    plt.savefig(DOSSIER + "/densites.png")
    plt.close()


plot_energie()
plot_viriel()
plot_rayons()
plot_histo_densite_plusieurs()
plot_axes()
