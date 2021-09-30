#!/bin/bash

QUAIL=/home/ubuntu/quail/
chmod 400 ${QUAIL}/data/aws01.pem
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

count=$((count-1))

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
	(ssh -i ${QUAIL}/data/aws01.pem ubuntu@${ip} "${QUAIL}/scripts/run_decrypt.sh $count") &
	fi
done < ${QUAIL}/scripts/committee.txt

wait

