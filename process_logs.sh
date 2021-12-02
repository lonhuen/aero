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

echo "Prover CPU (s)"
#grep -h -r "Prover CPU Time" ${LOG_DIR}
grep -h -r "Prover CPU Time" ${LOG_DIR} | awk '{total += $4; count++} END {print total/count}'

echo "Verifier CPU (s)"
#grep -h -r "Verifier CPU Time" ${LOG_DIR}
grep -h -r "Verifier CPU Time" ${LOG_DIR} | awk '{total += $4; count++} END {print total/count}'

if [ "$flag" = true ]; then
	echo "Committee Offline CPU(s)"
	##grep -h -r "real" ${LOG_DIR}/*/committee_offline*.log
	### get the first, since it might run the second round of offline
	user=$(grep -h -m 1 -r "user" ${LOG_DIR}/*/committee_offline*.log  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}')
	sys=$(grep -h -m 1 -r "sys" ${LOG_DIR}/*/committee_offline*.log  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}')
	echo "$user + $sys" | bc
fi
if [ "$flag" = true ]; then
	echo "Committee Online CPU(s)"
	##grep -h -r "real" ${LOG_DIR}/*/committee_online*.log
	user=$(grep -h -m 1 -r "user" ${LOG_DIR}/*/committee_online*.log  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}')
	sys=$(grep -h -m 1 -r "sys" ${LOG_DIR}/*/committee_online*.log  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}')
	echo "$user + $sys" | bc
fi

echo "Prover Network (B)"
grep -h -m 1 -r "sent bytes" ${LOG_DIR}/*/total.log  | awk '{total += $3; count++} END {printf("%f\n",total/(count * 15))}'
#echo "Prover Latency (s)"

echo "Verifier Network (B)"
grep -h -m 1 -r "recv bytes" ${LOG_DIR}/*/total.log  | awk '{total += $3; count++} END {printf("%f\n",total/(count * 15))}'

if [ "$flag" = true ]; then
	echo "Committee Offline Network (B)"
	grep -h -m 1 -r "sent bytes" ${LOG_DIR}/*/committee_offline${IID}.log  | awk '{printf("%f\n",$3)}'

	echo "Committee Online Network (B)"
	grep -h -m 1 -r "sent bytes" ${LOG_DIR}/*/committee_online${IID}.log  | awk '{printf("%f\n",$3)}'


	echo "Committee Offline Latency(s)"
	#grep -h -m 1 -r "real" ${LOG_DIR}/*/committee_online*.log 
	grep -h -m 1 -r "real" ${LOG_DIR}/*/committee_online*.log  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}'
fi

train_model=$(grep -h -r "End:     train model" ${LOG_DIR} | sed 's|[^0-9]*\([0-9\.]*\)|\1 |g' | awk '{total += $1; count++} END {printf("%f\n",total/count)}')
upload_data=$(grep -h -r "End:     upload data" ${LOG_DIR} | sed 's|[^0-9]*\([0-9\.]*\)|\1 |g' | awk '{total += $1; count++} END {printf("%f\n",total/count)}')

verify=0
count=0
lines=$(grep -h -r "End:     verify the data" ${LOG_DIR})
while read LINE; do
	tmp=$(echo $LINE | sed 's|[^0-9]*\([0-9\.]*\)|\1 |g')
	unit=${LINE: -2}
	if [ "$unit" = "µs" ]; then
		verify=$(echo "$verify + $tmp/1000000" | bc -l)
		#echo "$verify $tmp $unit"
	elif [ "$unit" = "ms" ]; then
		verify=$(echo "$verify + $tmp/1000" | bc -l)
		#echo "$verify $tmp $unit"
	else
		verify=$(echo "$verify + $tmp" | bc -l)
		#echo "$verify $tmp ${unit: -1}"
	fi
	count=$((count+1))
done <<< "$(echo -e "$lines")"
#vunit=$(grep -h -r "End:     verify the data" ${LOG_DIR} | head -1)
echo "Prover Latency(s)"
echo "$train_model + $upload_data - 1" | bc

echo "Verifier Latency"
echo "$verify/$count.0" | bc -l
