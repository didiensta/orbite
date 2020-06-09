#!/bin/python3
import sys
import numpy as np

def gaussian(nb):
    if len(sys.argv) < 5:
        sys.exit("Requiert une taille sigma pour la gaussienne en 4e argument")
    try:
        sigma = float(sys.argv[4])
    except ValueError:
        sys.exit("Le 4e argument une taille sigma pour la gaussienne (float)")
    else:
        return np.random.normal(scale=sigma, size=(nb,6))

models = {
    "gaussian": gaussian
}

if __name__ == "__main__":

    # Lecture des arguments
    if len(sys.argv) < 4:
        sys.exit("Requiert au moins 3 arguments")

    output = sys.argv[1]

    try:
        nb = int(sys.argv[2])
    except ValueError:
        sys.exit("Le 2e argument doit être le nombre de particule (int)")

    if sys.argv[3] in models.keys():
        mod = models[sys.argv[3]]
    else:
        sys.exit("Le 3e arguments doit être dans " + str(list(models.keys())))

    distrib = mod(nb)
    np.savetxt(output, distrib, delimiter=';')