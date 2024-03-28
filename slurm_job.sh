#!/bin/bash
#
#SBATCH --job-name=index-conjecture
#SBATCH --ntasks-per-node=64
#SBATCH -N=4

srun ./client
