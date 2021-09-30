#!/bin/bash

N_PLAYERS=$1
THRESHOLD=$2  # Require 2*THRESHOLD < N_PLAYERS

QUAIL=/home/ubuntu/quail

echo 3
echo RootCA

echo $N_PLAYERS

cat ${QUAIL}/scripts/committee.txt | while read smryline
do
linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	for i in ${linearray[@]:1}; do
		echo ${linearray[0]}
		echo "Player$i.crt"
        done
done
#for (( i = 0; i < $N_PLAYERS; i++ ))
#do
#  echo 127.0.0.1
#  echo Player$i.crt
#done
#echo 172.31.47.222
#echo Player$((N_PLAYERS-1)).crt

echo N
echo N
echo 2
echo 300424569129657234489620267994584186881

echo $THRESHOLD
