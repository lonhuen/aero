#!/bin/bash

QUAIL=/home/ubuntu/quail

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
		ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ubuntu@${ip} "sudo cp -r /root/SCALE-MAMBA/Data /home/ubuntu"&
	fi
done < ${QUAIL}/scripts/committee.txt

wait
	
