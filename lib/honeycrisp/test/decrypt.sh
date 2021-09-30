#!/bin/bash
[[ $UID = 0 ]] || exec sudo $0 "$@"

if [ -z $1 ]; then
	echo "provide # of instances"
	exit
fi
j=$1

N_1=$((j+1))
N_2=$((j+1))
THRESHOLD=1

#don't change servers for now
#python chooseSubset.py ${N_1} ${N_2} > ./Data/subset.txt
#echo 'Subset chosen:'
#cat ./Data/subset.txt
#
###./renameShares.sh ${N_2} ./Data ./Data/subset.txt
sed -i "8s/.*/k = $j/" Programs/decrypt/decrypt.mpc
./compile.py Programs/decrypt # hack to compile decrypt for later
DIR="./Data"

for (( i=0; i< ${N_2}; i++ ))
do
  j=$i
  sed 's/'${j}'/'${i}'/' ${DIR}/Player${j}_shareout.txt > ${DIR}/Player${i}_sharein.txt
done

N_PLAYERS=$N_2

perl -E 'print "1\n", "1\n", "1\n"' > ./Data/Player$(($N_PLAYERS - 1))\_in.txt

cat /dev/null > publicin.txt
for i in `seq 8192`
do 
  echo "1" >> publicin.txt
done

echo 'Measuring the runtime and communication cost of decrypt'

### TODO scp the Data directoy also the publicin.txt to all the peer machines
### TODO scp the keygen.mpc to all the peer machines
### TODO pssh to each peer to run the player.x
for (( i = 0; i <= $(($N_PLAYERS - 2)); i++ ))
do
	(cat publicin.txt | ./Player.x $i Programs/decrypt > /dev/null 2> /dev/null) &
done

COMM_T0=$(cat /proc/net/dev | grep -o lo..\[0-9\]* | grep -o \[0-9\]*)
#COMM_T0=$(cat /proc/net/dev | grep -o eth0..\[0-9]\* | grep -o \[0-9\]*)
time (cat publicin.txt | ./Player.x  $((N_PLAYERS - 1)) Programs/decrypt > /dev/null 2> /dev/null)


#COMM_T1=$(cat /proc/net/dev | grep -o eth0..\[0-9]\* | grep -o \[0-9\]*)
COMM_T1=$(cat /proc/net/dev | grep -o lo..\[0-9]\* | grep -o \[0-9\]*)
echo 'Communication Cost (bytes):' $((COMM_T1 - COMM_T0))
