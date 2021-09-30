#!/bin/bash

QUAIL=/home/ubuntu/quail
HONEYCRISP=${QUAIL}/lib/honeycrisp
SCALE_MAMBA=/root/SCALE-MAMBA

[[ $UID = 0 ]] || exec sudo $0 "$@"

# update scripts first
bash ${QUAIL}/scripts/update_honeycrisp.sh

cd ${SCALE_MAMBA}
bash ${SCALE_MAMBA}/decrypt.sh $1

echo 'Measuring the runtime and communication cost of decrypt'

### TODO scp the Data directoy also the publicin.txt to all the peer machines
### TODO scp the keygen.mpc to all the peer machines
### TODO pssh to each peer to run the player.x

#COMM_T0=$(cat /proc/net/dev | grep -o eth0..\[0-9]\* | grep -o \[0-9\]*)
ip4=$(/sbin/ip -o -4 addr list eth0 | awk '{print $4}' | cut -d/ -f1)

cat ${QUAIL}/scripts/committee.txt | while read smryline
do
linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
len=${#linearray[@]}
len=$((len-1))
if [ "${ip4}" = "${linearray[0]}" ]; then
	for i in ${linearray[@]:1:$len}; do
		#./Player.x $i Programs/keygen > /dev/null 2> /dev/null &
		#(cat publicin.txt | ./Player.x $i Programs/decrypt > /dev/null 2> /dev/null) &
		(cat publicin.txt | ./Player.x $i Programs/decrypt | tee run$i.log) &
        done
fi
done

#time(cat publicin.txt | ./Player.x $len Programs/decrypt > /dev/null 2> /dev/null)
time(cat publicin.txt | ./Player.x $len Programs/decrypt)
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

#COMM_T1=$(cat /proc/net/dev | grep -o lo..\[0-9]\* | grep -o \[0-9\]*)
#echo 'Communication Cost (bytes):' $((COMM_T1 - COMM_T0))

