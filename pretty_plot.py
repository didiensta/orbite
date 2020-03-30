"""
Plot the evolution of energy, axes ratios, virial, rayons, 
and density profile.
The first argument is the folder where all data is saved, 
the second one is the .ini configuration file of the simulation.
"""
import sys
import os
import struct
import configparser

FOLDER = sys.argv[1]
INI_FILE = sys.argv[2]

# Read the number of saved particules from the .ini file
config = configparser.ConfigParser()
config.read(INI_FILE)
nb_part_save = int(config['Parameters']['nb_particules_save'])

statinfo = os.stat(FOLDER + "/data").st_size
print(statinfo)

with open(FOLDER + "/data", 'rb') as binary_file:
    binary_data = binary_file.read()

    # specify the formating structure of binary data
    format = "dI" + "ddd" * 2 * nb_part_save + "dddddddddddddddd"

    iterator = struct.iter_unpack(format, binary_data)

    for unpacked_data in iterator:
        print(unpacked_data)
        print("\n")
