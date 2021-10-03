#!/bin/bash

QUAIL=/home/ubuntu/quail

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
		ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ubuntu@${ip} "sudo rm -r -f /root/SCALE-MAMBA/Data; sudo cp -r /home/ubuntu/Data /root/SCALE-MAMBA/"&
	fi
done < ${QUAIL}/scripts/committee.txt

wait
	
