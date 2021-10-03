#!/bin/bash

QUAIL=/home/ubuntu/quail/
count=0
while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
		len=${#linearray[@]}
		count=$((count+len-1))
	fi
done < ${QUAIL}/scripts/committee.txt

THRESHOLD=$((count*2/5))
count=$((count-1))

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	scp ${QUAIL}/lib/honeycrisp/source/decrypt.mpc -i ${QUAIL}/data/aws01.pem ubuntu@${ip}:${QUAIL}/lib/honeycrisp/source/decrypt.mpc
	#/home/ubuntu/quail/lib/honeycrisp/source/decrypt.mpc
	fi
done < ${QUAIL}/scripts/committee.txt

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
	#scp ${QUAIL}/scripts/committee.txt ubuntu@${ip}:${QUAIL}/scripts/committee.txt
	(ssh ubuntu@${ip} -i ${QUAIL}/data/aws01.pem "${QUAIL}/scripts/run_decrypt.sh $count $THRESHOLD") &
	fi
done < ${QUAIL}/scripts/committee.txt


wait

