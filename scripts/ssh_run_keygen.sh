#!/bin/bash

QUAIL=/home/ubuntu/quail
count=0
chmod 400 ${QUAIL}/data/aws01.pem
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

#THRESHOLD=$((count*2/5))
THRESHOLD=2
echo $THRESHOLD
count=$((count-1))

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
	scp -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ${QUAIL}/scripts/committee.txt ubuntu@${ip}:${QUAIL}/scripts/committee.txt
	scp -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ${QUAIL}/lib/honeycrisp/test/keygen.sh ubuntu@${ip}:${QUAIL}/lib/honeycrisp/test/keygen.sh
	scp -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ${QUAIL}/scripts/run_keygen.sh ubuntu@${ip}:${QUAIL}/scripts/run_keygen.sh
	fi
done < ${QUAIL}/scripts/committee.txt

while read smryline
do
	linearray=($(awk -F, '{$1=$1} 1' <<<"${smryline}"))
	ip=${linearray[0]}
	if [[ $ip = *[!\ ]* ]]; then
	# ssh to ip to run the content
	(ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -i ${QUAIL}/data/aws01.pem ubuntu@${ip} "${QUAIL}/scripts/run_keygen.sh $count $THRESHOLD" 2>&1 | tee ssh$ip.log)&
	fi
done < ${QUAIL}/scripts/committee.txt

wait
	
