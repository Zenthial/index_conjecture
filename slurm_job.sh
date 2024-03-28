#!/bin/bash
#
#SBATCH --job-name=index-conjecture
#SBATCH --ntasks-per-node=64
#SBATCH --nodes=1
#SBATCH --time=2:00:00:00

srun ./client
