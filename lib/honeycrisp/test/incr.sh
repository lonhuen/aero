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
THRESHOLD=1

rm ./Data/*
seq ${N_1} > ./Data/evalPoints.txt
./genSetupOptions.sh ${N_1} ${THRESHOLD} | ./Setup.x > /dev/null

### TODO scp the Data directoy to all the peer machines
### TODO scp the keygen.mpc to all the peer machines
### TODO pssh to each peer to run the keygen.sh with ID
./benchmark.sh ./Programs/keygen/ ${N_1} 2 $(($N_PLAYERS - 1))
# ssh remote addr run ./benchmark.sh ./Programs/keygen/ ${N_1} 2 $PlayerID
# ...

sed -i "8s/.*/k = $j/" Programs/decrypt/decrypt.mpc
###./testd.sh $((j+1)) $((j+1)) 1

#don't change servers for now
#python chooseSubset.py ${N_1} ${N_2} > ./Data/subset.txt
#echo 'Subset chosen:'
#cat ./Data/subset.txt
#
###./renameShares.sh ${N_2} ./Data ./Data/subset.txt
DIR="./Data"
SUBSET="./Data/subset.txt"

for (( i=0; i< ${N_2}; i++ ))
do
  #j=$( sed $((${i} + 1))'q;d' ${SUBSET} )
  # TODO ssh to j-th peer to rename its data
  j=$i
  sed 's/'${j}'/'${i}'/' ${DIR}/Player${j}_shareout.txt > ${DIR}/Player${i}_sharein.txt
done

N_PLAYERS=$N_2

#./genSetupOptions.sh ${N_2} ${THRESHOLD} | ./Setup.x > /dev/null

perl -E 'print "1\n", "1\n", "1\n"' > ./Data/Player$(($N_PLAYERS - 1))\_in.txt

for i in `seq 4096`
do 
  echo "1" >> publicin.txt
done

### TODO scp the Data directoy also the publicin.txt to all the peer machines
### TODO scp the keygen.mpc to all the peer machines
### TODO pssh to each peer to run the player.x
for (( i = 0; i <= $(($N_PLAYERS - 2)); i++ ))
do
  ./Player.x $i Programs/decrypt > /dev/null 2> /dev/null &
done

COMM_T0=$(cat /proc/net/dev | grep -o lo..\[0-9\]* | grep -o \[0-9\]*)
#COMM_T0=$(cat /proc/net/dev | grep -o eth0..\[0-9]\* | grep -o \[0-9\]*)
time ( cat publicin.txt | ./Player.x  $((N_PLAYERS - 1)) Programs/decrypt > /dev/null 2> /dev/null ) 


#COMM_T1=$(cat /proc/net/dev | grep -o eth0..\[0-9]\* | grep -o \[0-9\]*)
COMM_T1=$(cat /proc/net/dev | grep -o lo..\[0-9]\* | grep -o \[0-9\]*)
echo 'Communication Cost (bytes):' $((COMM_T1 - COMM_T0))
