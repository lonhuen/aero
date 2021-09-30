#!/bin/bash

if [ -z $1 ]; then
	echo "provide # of instances"
	exit
fi


QUAIL=/home/ubuntu/quail
HONEYCRISP=${QUAIL}/lib/honeycrisp
SCALE_MAMBA=/root/SCALE-MAMBA

[[ $UID = 0 ]] || exec sudo $0 "$@"

# update scripts first
bash ${QUAIL}/scripts/update_honeycrisp.sh

cd ${SCALE_MAMBA}
bash ${SCALE_MAMBA}/keygen.sh $1

reqs=$(./compile.py Programs/keygen | grep "Program requires:")
#./compile.py Programs/decrypt # hack to compile decrypt for later

echo $reqs

N_TRIPLES=$(echo $reqs | grep -o \'triple\'\)..\[0-9\]* | grep -o \[0-9\]*)
N_BITS=$(echo $reqs | grep -o \'bit\'\)..\[0-9\]* | grep -o \[0-9\]*)
N_SQUARES=$(echo $reqs | grep -o \'square\'\)..\[0-9\]* | grep -o \[0-9\]*)

if [[ $N_TRIPLES == '' ]]
then
  N_TRIPLES=1  # 1 instead of 0 since 0 represents infinity
fi

if [[ $N_BITS == '' ]]
then
  N_BITS=1
fi

if [[ $N_SQUARES == '' ]]
then
  N_SQUARES=1
fi

echo 
echo 'Measuring the runtime and communication cost of keygen' 

#COMM_T0=$(cat /proc/net/dev | grep -o lo..\[0-9\]* | grep -o \[0-9\]*)
ip4=$(/sbin/ip -o -4 addr list eth0 | awk '{print $4}' | cut -d/ -f1)

cat ${QUAIL}/scripts/committee.txt | while read smryline
do
linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
len=${#linearray[@]}
len=$((len-1))
if [ "${ip4}" = "${linearray[0]}" ]; then
	for i in ${linearray[@]:1:$len}; do
		./Player.x $i Programs/keygen >/dev/null 2> /dev/null &
        done
fi
done
time(./Player.x -max ${N_TRIPLES},${N_SQUARES},${N_BITS} -maxI ${N_IO} ${linearray[${len}]} Programs/keygen >/dev/null 2> /dev/null)
echo "Done"

#for (( i = 0; i <= $(($N_PLAYERS - 2)); i++ ))
#do
#  ./Player.x -max ${N_TRIPLES},${N_SQUARES},${N_BITS} -maxI ${N_IO} $i Programs/keygen > /dev/null 2> /dev/null &
#done
#
#time (./Player.x -max ${N_TRIPLES},${N_SQUARES},${N_BITS} -maxI ${N_IO} $(($N_PLAYERS - 1)) Programs/keygen > /dev/null 2> /dev/null ) 
#
#COMM_T1=$(cat /proc/net/dev | grep -o lo..\[0-9]\* | grep -o \[0-9\]*)
#echo 'Communication Cost (bytes):' $((COMM_T1 - COMM_T0))
