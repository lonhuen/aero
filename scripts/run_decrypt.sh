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

ip4=$(/sbin/ip -o -4 addr list eth0 | awk '{print $4}' | cut -d/ -f1)

start=$(cat /proc/net/dev | grep "eth0")
in_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $2 }')
out_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $10 }')
COMM_T0=$(cat /proc/net/dev | grep -o lo..\[0-9\]* | grep -o \[0-9\]*)

cat ${QUAIL}/scripts/committee.txt | while read smryline
do
linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
if [ "${ip4}" = "${linearray[0]}" ]; then
	len=${#linearray[@]}
	last=$((len-1))
	len=$((last-1))
	for i in ${linearray[@]:1:$len}; do
		#./Player.x $i Programs/keygen > /dev/null 2> /dev/null &
		#(cat publicin.txt | ./Player.x $i Programs/decrypt > /dev/null 2> /dev/null) &
		#(cat publicin.txt | ./Player.x $i Programs/decrypt | tee run$i.log) &
		(cat publicin.txt | ./Player.x $i Programs/decrypt > /dev/null 2> /dev/null) &
        done
	#time(cat publicin.txt | ./Player.x $len Programs/decrypt > /dev/null 2> /dev/null)
	time(cat publicin.txt | ./Player.x ${linearray[$last]} Programs/decrypt > /dev/null 2> /dev/null)
fi
done

end=$(cat /proc/net/dev | grep "eth0")
in_bytes_end=$(echo $end| awk -v OFS=, '/eth0:/ { print $2 }')
out_bytes_end=$(echo $end | awk -v OFS=, '/eth0:/ { print $10 }')
echo "recv bytes from eth0" $((in_bytes_end - in_bytes)) 
echo "sent bytes from eth0" $((out_bytes_end - out_bytes)) 

COMM_T1=$(cat /proc/net/dev | grep -o lo..\[0-9]\* | grep -o \[0-9\]*)
echo 'Communication Cost (bytes) from lo:' $((COMM_T1 - COMM_T0))
