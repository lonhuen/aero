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
	scp -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ${QUAIL}/scripts/committee.txt ubuntu@${ip}:${QUAIL}/scripts/committee.txt
	fi
done < ${QUAIL}/scripts/committee.txt

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
	(ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ubuntu@${ip} "${QUAIL}/scripts/run_keygen.sh $count" 2>&1 | tee ssh$ip.log)&
	fi
done < ${QUAIL}/scripts/committee.txt

wait
	
