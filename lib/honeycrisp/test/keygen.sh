#!/bin/bash
[[ $UID = 0 ]] || exec sudo $0 "$@"

if [ -z $1 ]; then
	echo "provide # of instances"
	exit
fi
j=$1

sed -i "11s/.*/k = $j/" Programs/keygen/keygen.mpc

N_1=$((j+1))
N_2=$((j+1))
THRESHOLD=$2
if [ -z $2 ]; then
	THRESHOLD=1
fi

rm ./Data/*
seq ${N_1} > ./Data/evalPoints.txt
./genSetupOptions.sh ${N_1} ${THRESHOLD} | ./Setup.x > /dev/null
#./genSetupOptions.sh ${N_1} ${THRESHOLD}

#./benchmark.sh ./Programs/keygen/ ${N_1} 2 $((N_PLAYERS - 1))
