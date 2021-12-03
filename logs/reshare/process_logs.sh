#! /bin/bash
LOG_DIR=$1
IID=$2

if [ ! -d ${LOG_DIR} ]; then
	echo "give a log directory"
	exit
fi

flag=true
#if [[ "${LOG_DIR}" == *"baseline"* ]]; then
#	flag=false
#fi

echo "Committee Reshare CPU(s)"
##grep -h -r "real" ${LOG_DIR}/*/committee_offline*.log
### get the first, since it might run the second round of offline
user=$(grep -h -m 1 -r "user" ${LOG_DIR}/committee_reshare*.log  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}')
sys=$(grep -h -m 1 -r "sys" ${LOG_DIR}/committee_reshare*.log  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}')
echo "$user + $sys" | bc

echo "Committee Reshare Network (B)"
grep -h -m 1 -r "sent bytes" ${LOG_DIR}/committee_reshare27.log  | awk '{print $3}'
#echo "Prover Latency (s)"
