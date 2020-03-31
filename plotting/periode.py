"""
Analyse les orbites de la simulation
plot log(a)~log(periode) et log(periode)~log(-E)
fait les regressions lineaires
les barres d'erreurs represente un ecart type
"""
import sys
import os
import glob
import math
import csv
import peakutils
import warnings
import numpy as np
import matplotlib
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
matplotlib.use('Agg')
warnings.filterwarnings("ignore")

dossier = sys.argv[1]

def temps_dynamique():
    """ retourne une np array du temps de chaque pas """
    temps = []
    with open(dossier + "/infos.csv") as csvDataFile:
        csvReader = csv.reader(csvDataFile, delimiter=';')
        for row in csvReader:
            t = float(row[0])
            temps.append(t)
    return np.array(temps)

def periode(nb):
    """ fait toute l'analyse et les plots
        nb : nombre de particule a analyser """

#lecture des fichiers
    listcsv = sorted(glob.glob(dossier + "/positions/*.csv"), key=os.path.getmtime)
    nbfile = len(listcsv)
    particules_pos = np.zeros((nb, nbfile, 3))
    particules_energie = np.zeros((nb, nbfile))
    x = temps_dynamique()
    f = 0
    for file in listcsv:
        with open(file) as csvDataFile:
            csvReader = csv.reader(csvDataFile, delimiter=';')
            i = 0
            for row in csvReader:
                #x
                particules_pos[i][f][0] = float(row[0])
                #y
                particules_pos[i][f][1] = float(row[1])
                #z
                particules_pos[i][f][2] = float(row[2])
                #energie cinetic+potential
                particules_energie[i][f] = float(row[3]) + float(row[4])
                i = i +1
                if (i == nb):
                    break
        f = f +1


    #liste des periodes
    P = []
    #liste des energies
    E = []
    #liste des ecarts types sur la periode
    SigmaP = []
    #liste des ecarts types sur l'energie
    SigmaE = []
    #liste des appocentres
    RA = []
    #liste des pericentres
    RP = []
    #listes des ecarts types de RA et RP
    SigmaRA = []
    SigmaRP = []

    #compteur du nombre de particule non rejetees
    nbr_accepte = 0

    #parametre pour la detection des pics (ajuster si besoin)
    THRES = 0.01
    MIN_DIST = 1
    
    #temps dynamique a partir duquel on commence a regarder les periodes
    #(pour enlever l'effondrement)
    START_TIME = 75

    #taux d'ecart maximum tolere de l'amplitude par rapport a l'amplitude moyenne
    TOL_AMP = 0.2


    for id in range(0,nb):

        x = temps_dynamique()
        #distance au centre
        y = []
        for p in (particules_pos[id]):
            y.append(math.sqrt(p[0]**2 + p[1]**2 + p[2]**2))
        y = np.array(y)

        energies = np.array(particules_energie[id])

        fig, axes = plt.subplots(2, 1)

        #plot l'evolution de la distance au centre 
        axes[0].plot(x, y)
        axes[0].set_title(r"$|r(t)|$")

        #plot energie
        axes[1].plot(x, (energies-energies[0])/energies[0] * 100)
        axes[1].set_title("variation energy %")


#on coupe le debut pour ignorer l'effondrement
        indexstart = 0 
        for i in range(0,len(x)):
            if(x[i]< START_TIME):
                indexstart = i
                break

        y = y[indexstart :-1]
        x = x[indexstart :-1]
        energies = energies[indexstart :-1]

#calcul des pics superieurs (maximum locaux) sur tout l'intervalle de temps
        #liste des indexes des pics
        indexes = peakutils.indexes(y, thres=THRES, min_dist=MIN_DIST)

        #liste des temps des maximum 
        picsXmax = [x[i] for i in indexes]
        #liste de la hauteur des maximum 
        picsYmax = [y[i] for i in indexes]

        if( len(indexes) == 0):
            print(str(id) + " pas de pics detecte")
            continue;

#recherche d'un intervalle de temps ou l'amplitude des pics max varie "peu"
        amp_min = picsYmax[0]
        amp_max = picsYmax[0]

        indexdebut = [0]
        indexfin = [0]
        for i in range(0, len(picsYmax)):
            amp_min = min(amp_min, picsYmax[i])
            amp_max = max(amp_max , picsYmax[i])
            if( abs((amp_max-amp_min)/ ((amp_max+amp_min)/2.)) > TOL_AMP ):
                indexdebut.append(i)
                indexfin.append(i)
                amp_min = picsYmax[i]
                amp_max = picsYmax[i]
                continue
            indexfin[-1] = i
        
        #on cherche l'intervalle de temps le plus long possible
        idmaxinter = 0
        maxinter = indexfin[0]-indexdebut[0]
        for i in range(0,len(indexdebut)):
            if( indexfin[i]-indexdebut[i] > maxinter):
                maxinter = indexfin[i]-indexdebut[i]
                idmaxinter = i

        #on coupe pour ne garder que le plus grand interval
        debut = indexes[indexdebut[idmaxinter]]
        fin = indexes[indexfin[idmaxinter]]
        if( fin == debut):
            print(str(id) + " pas de bon intervalle")
            continue
        y = y[debut:fin]
        x = x[debut:fin]
        energies = energies[debut:fin]

#calcul des pics (min et max) sur le sous intervalle choisit

        #calcul des maximum
        indexes = peakutils.indexes(y, thres=THRES, min_dist=MIN_DIST)
        #liste des temps des pics maximum
        picsXmax = [x[i] for i in indexes]
        #liste des valeurs des pics maximum ( = r(picsXmax) )
        picsYmax = [y[i] for i in indexes]
        axes[0].plot(picsXmax, picsYmax, marker="o", ls="", ms=3)

        #calcul des minimum
        indexes = peakutils.indexes(-y, thres=THRES, min_dist=MIN_DIST)
        picsXmin = [x[i] for i in indexes]
        picsYmin = [y[i] for i in indexes]
        axes[0].plot(picsXmin, picsYmin, marker="o", ls="", ms=3)

        #calcul de l'energie moyenne / maximum / minimum
        emoy = np.sum(energies)/len(energies)
        emax = np.max(energies)
        emin = np.min(energies)

        ##calcul du taux d'ecart a la moyenne de l'energie pour contenir 
        ##toutes les valeurs de l'energie sur l'intervalle
        ##TODO : faire le calcul de l'ecart type a la place ...
        #ecartmax = max( abs( (emax-emoy)/emoy) , abs((emoy-emin)/emoy))
        #plt.text(0.75,0.75,"+/- " + str(ecartmax*100) + "%",horizontalalignment='center',
        # verticalalignment='center',
        # transform=axes[1].transAxes)

        #sauvegarde le plot des deux graphes : r(t) et energie(t)
        plt.savefig(dossier + "/periodes/" +  "/periode_" + str(id) + ".png")
        plt.close()

#rejet des mauvaises particules

        #liste des intervalles de temps entre les maximum
        intervalmax = np.array(picsXmax[1:-1]) - np.array(picsXmax[0:-2])
        #liste des intervalles de temps entre les minimum
        intervalmin = np.array(picsXmin[1:-1]) - np.array(picsXmin[0:-2])

        #rejet si moins de 5 periodes
        if(len(intervalmax)< 5 or len(intervalmin)< 5):
            print(str(id) + " pas de bon intervalle")
            continue

        #periode moyenne pour les maximum
        periode_max_moy = np.sum(intervalmax)/len(intervalmax)
        #periode moyenne pour les minimum
        periode_min_moy = np.sum(intervalmin)/len(intervalmin)

        #rejet si il existe des interval entre deux max ou min
        #trop eloigne de la moyenne
        #Dans ce cas c'est que la detection automatique des pics a echouee
        #(il nous manque un pic ou il y en a en trop)
        if( len([ I for I in intervalmax if I > 1.5*periode_max_moy]) > 0 ):
            print(str(id) + " mauvaise detection des pics : pics en moins")
            continue
        if( len([ I for I in intervalmax if I < 0.7*periode_max_moy]) > 0 ):
            print(str(id) + " mauvaise detection des pics : pics en moins")
            continue

        if( len([ I for I in intervalmin if I > 1.5*periode_min_moy]) > 0 ):
            print(str(id) + " mauvaise detection des pics : pics en moins")
            continue
        if( len([ I for I in intervalmin if I < 0.7*periode_min_moy]) > 0 ):
            print(str(id) + " mauvaise detection des pics : pics en moins")
            continue

        #ecarttype_periode = np.sqrt(np.sum((interval-periode_moy)**2)/len(interval))
        #ecarttype_energie = np.sqrt(np.sum((energies-emoy)**2)/len(y))
        picmaxmax = np.max(picsYmax)
        picmaxmin = np.min(picsYmax)
        picminmax = np.max(picsYmin)
        picminmin = np.min(picsYmin)
        picmaxmoy = np.sum(picsYmax)/len(picsYmax)
        picminmoy = np.sum(picsYmin)/len(picsYmin)

        #rejet si la periode trouve avec les max et les min est trop differente
        if( abs( 1 - periode_min_moy / periode_max_moy) > 0.01):
            print(str(id) + " moyennes pics min et max trop eloignees")
            continue
        periode_moy = 0.5*(periode_max_moy + periode_min_moy)

#on ajoute toutes les valeurs de cette particule aux listes principales 
        P.append(periode_moy)
        E.append(emoy)

        log_emoy = np.sum(np.log(-energies))/len(energies)
        log_ecarttype_energie = np.sqrt(np.sum((np.log(-energies)-log_emoy)**2)/len(y))

        log_periode_max_moy = np.log(periode_max_moy)
        log_ecarttype_periode_max = np.sqrt(np.sum(
            (np.log(intervalmax)-log_periode_max_moy)**2)
            /len(intervalmax))

        log_periode_min_moy = np.log(periode_min_moy)
        log_ecarttype_periode_min = np.sqrt(np.sum(
            (np.log(intervalmin)-log_periode_min_moy)**2)
            /len(intervalmin))

        log_ecarttype_periode = np.sqrt(1./4.*(log_ecarttype_periode_max**2 + log_ecarttype_periode_min**2)) 

        SigmaP.append(log_ecarttype_periode)
        SigmaE.append(log_ecarttype_energie)

        RA.append(picmaxmoy)
        RP.append(picminmoy)

        ecarttype_picmax = np.sqrt( np.sum( (picsYmax - picmaxmoy )**2) / len(picsYmax) )
        ecarttype_picmin = np.sqrt( np.sum( (picsYmin - picminmoy )**2) / len(picsYmin) )
        SigmaRA.append(ecarttype_picmax)
        SigmaRP.append(ecarttype_picmin)

        nbr_accepte = nbr_accepte + 1


        #plot l'orbite de la particule en 3D
        xs = []
        ys = []
        zs = []
        for p in (particules_pos[id]):
            xs.append(p[0])
            ys.append(p[1])
            zs.append(p[2])
        fig = plt.figure()
        ax = fig.add_subplot(111, projection='3d')
        ax.plot(xs, ys, zs)
        plt.savefig(dossier + "/periodes/" +  "/orbite_" + str(id) + ".png")

    print("taux d'acceptation : " + str(nbr_accepte/float(nb)))
    log_P = np.log(P)
    log_E = np.log(-np.array(E))
    RA = np.array(RA)
    RP = np.array(RP)
    SigmaRA = np.array(SigmaRA)
    SigmaRP = np.array(SigmaRP)


#on plot les graphs et on fait le fit lineaire        

    #pour prendre 20 grosses particules qui representent chaque interval de periodes

    INTERVALLE_P = np.linspace( np.min(log_P), np.max(log_P), 20)
    INTERVALLE_P = [ [INTERVALLE_P[i],INTERVALLE_P[i+1]] for i in range(0, len(INTERVALLE_P) -1) ]
    Big_log_P = []
    Big_log_E = []
    Big_RA = []
    Big_RP = []
    Big_SigmaP = []
    Big_SigmaE = []
    Big_SigmaRA = []
    Big_SigmaRP = []

    for I in INTERVALLE_P:
        p = 0
        e = 0
        ra = 0
        rp = 0
        sigma_ra = []
        sigma_rp = []
        sigma_p = []
        sigma_e = []
        c = 0.
        for i in range(0, len(log_P)):
            if(log_P[i] > I[0] and log_P[i]< I[1]):
                p = p + log_P[i]
                e = e + log_E[i]
                ra = ra + RA[i]
                rp = rp + RP[i]
                sigma_ra.append(SigmaRA[i])
                sigma_rp.append(SigmaRP[i])
                sigma_p.append(SigmaP[i])
                sigma_e.append(SigmaP[i])

                c = c + 1.
        if( c>0):
            Big_log_P.append(p/c)
            Big_log_E.append(e/c)
            Big_RA.append(ra/c)
            Big_RP.append(rp/c)
            Big_SigmaP.append(np.sqrt(np.sum(np.array(sigma_p)**2)))
            Big_SigmaE.append(np.sqrt(np.sum(np.array(sigma_e)**2)))
            Big_SigmaRA.append(np.sqrt(np.sum(np.array(sigma_ra)**2)))
            Big_SigmaRP.append(np.sqrt(np.sum(np.array(sigma_rp)**2)))

    log_P  = np.array(Big_log_P)
    log_E  = np.array(Big_log_E)
    RA = np.array(Big_RA)
    RP = np.array(Big_RP)
    SigmaP = np.array(Big_SigmaP)
    SigmaE = np.array(Big_SigmaE)
    SigmaRA = np.array(Big_SigmaRA)
    SigmaRP = np.array(Big_SigmaRP)
        
#graph log(-E) log(periode)

    fig = plt.figure()
    plt.errorbar(log_P, log_E, SigmaP, SigmaE, fmt='.', ecolor='b', capthick=1)
    #plt.scatter(log_P,log_E,fmt='.',ecolor='b',capthick=1)

    ##fit normal
    #m,b= np.polyfit(log_P, log_E, 1)

    #fit avec un poids sur l'erreur
    x = log_P
    y = log_E
    w = np.array(SigmaE)**-2.
    delta = np.sum(w)*np.sum(w*x*x) - np.sum(w*x)**2. 
    m = 1./delta * ( np.sum(w)*np.sum(w*x*y) - np.sum(w*x)*np.sum(w*y))
    c = 1./delta * ( np.sum(w*x*x)*np.sum(w*y) - np.sum(w*x)*np.sum(w*x*y))
    sigma_m = np.sqrt( 1./delta * np.sum(w))
    sigma_c = np.sqrt( 1./delta * np.sum(w*x*x))

    print("pente log_P~log_E : " + str(m))
    print("sigma pente log_P~log_E : " + str(sigma_m))
    x = np.linspace(np.min(log_P), np.max(log_P), 1000)
    plt.plot(x, m*x + c, color = "red" )
    ax = plt.axes()
    plt.text(0.75,0.75,format(m, '.3f'),horizontalalignment='center',
     verticalalignment='center',
     transform=ax.transAxes, color="red")

    plt.xlabel(r"$\ln(\tau)$")
    plt.ylabel(r"$\ln(-E)$")
    plt.savefig(dossier + "/graph1" + ".png")
    plt.close()

#graph log(a) log(periode)

    #on cherche b entre 0 et 3  (a elargir si besoin)
    B = np.linspace(0, 3, 10000)
    Err = [] 
    M = [] 
    C = [] 
    Sigma_M = []
    Sigma_C = []

    
    w = np.array(SigmaP)**-2.
    #calcul de a pour differentes valeurs de b
    for b in B:
        A = 0.5*( np.sqrt(RA**2 + b**2) + np.sqrt(RP**2 + b**2) )
        log_A = np.log(A)
        x = np.log(A)
        y = log_P
        #m,c= np.polyfit(log_A,log_P, 1)

        #fit avec poids
        delta = np.sum(w)*np.sum(w*x*x) - np.sum(w*x)**2. 
        m = 1./delta * ( np.sum(w)*np.sum(w*x*y) - np.sum(w*x)*np.sum(w*y))
        c = 1./delta * ( np.sum(w*x*x)*np.sum(w*y) - np.sum(w*x)*np.sum(w*x*y))
        err = np.sum((log_P - (m*log_A+c))**2)/(np.sum(log_P**2))
        Err.append(err)

        M.append(m)
        C.append(c)
        sigma_m = np.sqrt( 1./delta * np.sum(w))
        sigma_c = np.sqrt( 1./delta * np.sum(w*x*x))
        Sigma_M.append(sigma_m)
        Sigma_C.append(sigma_c)
    
    #on selectionne la valeur de b qui minimise les residues
    Err = np.array(Err)
    M = np.array(M)
    C = np.array(C)
    err = np.min(Err)
    b = B[np.argmin(Err)]
    print("b : " + str(b))
    m = M[np.argmin(Err)]
    c = C[np.argmin(Err)]
    sigma_m = Sigma_M[np.argmin(Err)]
    sigma_c = Sigma_C[np.argmin(Err)]
    print("pente log_A~log_P : " + str(m))
    print("sigma pente log_A~log_P : " + str(sigma_m))
    A = 0.5*( np.sqrt(RA**2 + b**2) + np.sqrt(RP**2 + b**2) )
    SigmalogA = np.sqrt( (A**-1 * RA / (0.5* np.sqrt( RA**2 + b**2) ) )**2*SigmaRA**2
            + (A**-1 * RP / (0.5* np.sqrt( RP**2 + b**2) ) )**2*SigmaRP**2 ) 

    log_A = np.log(A)

    fig = plt.figure()

    plt.errorbar(log_A, log_P, SigmalogA, SigmaP,fmt='.', ecolor='b', capthick=1)
    #plt.scatter(log_A,log_P,fmt='.',ecolor='b',capthick=1)

    x = np.linspace(np.min(log_A), np.max(log_A), 10)
    plt.plot(x, m*x + c, color = "red" )
    ax = plt.axes()
    plt.text(0.75,0.25,format(m, '.3f'),horizontalalignment='center',
     verticalalignment='center',
     transform=ax.transAxes, color="red")

    plt.xlabel(r"$\ln(a)$")
    plt.ylabel(r"$\ln(\tau)$")
    plt.savefig(dossier + "/graph2" + ".png")
    plt.close()

    #on plot le graph de l'erreur suivant b 
    fig = plt.figure()
    plt.plot(B,Err)
    plt.xlabel("b")
    plt.ylabel("reste")
    plt.savefig(dossier + "/graph_b" + ".png")
    plt.close()




try:
    os.mkdir(dossier + "/periodes")
except:
  pass
periode(400)
