#! /bin/bash
LOG_DIR=$1
COMMITTEE=46
#CLIENT=32
#CLIENT=30

CLIENT=$(grep -h -r "nr_real" ${LOG_DIR}/config.yaml | awk '{print $2}')
grep -h -r "nr_real" ${LOG_DIR}/config.yaml
echo "Check that the total # of clients per instance is $CLIENT"
echo "Check that the # of committees per instance is $COMMITTEE"

if [ ! -d ${LOG_DIR} ]; then
	echo "give a log directory"
	exit
fi

echo "Prover CPU (s)"
grep -h -r "Prover CPU Time" ${LOG_DIR} | awk '{total += $4; count++} END {print total/count}'

echo "Verifier CPU (s)"
grep -h -r "Verifier CPU Time" ${LOG_DIR} | awk '{total += $4; count++} END {print total/count}'

echo "Committee Offline CPU(s)"
##grep -h -r "real" ${LOG_DIR}/*/committee_offline*.log
### get the first, since it might run the second round of offline
user=$(find ${LOG_DIR} -type f -name "committee_offline*.log" -print0 | xargs -0 grep -h -m 1 -r "user" | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m*60 + total_s}')
sys=$(find ${LOG_DIR} -type f -name "committee_offline*.log" -print0 | xargs -0 grep -h -m 1 -r "sys" | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m*60 + total_s}')
echo "($user + $sys)/$COMMITTEE" | bc -l

echo "Committee Online CPU(s)"
user=$(find ${LOG_DIR} -type f -name "committee_online*.log" -print0 | xargs -0 grep -h -m 1 -r "user" | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m*60 + total_s}')
sys=$(find ${LOG_DIR} -type f -name "committee_online*.log" -print0 | xargs -0 grep -h -m 1 -r "sys" | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m*60 + total_s}')
echo "($user + $sys)/$COMMITTEE" | bc -l

echo "Prover Network (B)"
#total_prover=$(grep -h -m 1 -r "sent bytes" ${LOG_DIR} --include "total.log"  | awk '{total += $3; count++} END {printf("%f\n",total/count)}')
total_prover=$(find ${LOG_DIR} -type f -name "total.log" -print0 | xargs -0 grep -h -m 1 -r "sent bytes" | awk '{total += $3} END {printf("%f\n",total)}')
echo "$total_prover/$CLIENT" | bc -l
#echo "Prover Latency (s)"

echo "Verifier Network (B)"
total_verifier=$(find ${LOG_DIR} -type f -name "total.log" -print0 | xargs -0 grep -h -m 1 -r "recv bytes" | awk '{total += $3} END {printf("%f\n",total)}')
echo "$total_verifier/$CLIENT" | bc -l

echo "Committee Offline Network (B)"
total_offline=$(find ${LOG_DIR} -type f -name "committee_offline$((COMMITTEE-1)).log" -print0 | xargs -0 grep -h -m 1 -r "sent bytes" | awk '{total += $3} END {printf("%f\n",total)}')
#total_offline=$(grep -h -m 1 -r "sent bytes" ${LOG_DIR} "committee_offline${COMMITTEE}.log"  | awk '{printf("%f\n",$3)}')
echo "$total_offline/$COMMITTEE" | bc -l

echo "Committee Online Network (B)"
#total_online=$(grep -h -m 1 -r "sent bytes" ${LOG_DIR} "committee_online${COMMITTEE}.log"  | awk '{printf("%f\n",$3)}')
total_online=$(find ${LOG_DIR} -type f -name "committee_online$((COMMITTEE-1)).log" -print0 | xargs -0 grep -h -m 1 -r "sent bytes" | awk '{total += $3} END {printf("%f\n",total)}')
echo "$total_online/$COMMITTEE" | bc -l

#echo "Committee Reshare Network (B)"
#grep -h -m 1 -r "sent bytes" ${LOG_DIR} "committee_reshare${IID}.log"  | awk '{printf("%f\n",$3)}'


## Measure Latency
#echo "Committee Offline Latency(s)"
##grep -h -m 1 -r "real" ${LOG_DIR}/*/committee_online*.log 
#grep -h -m 1 -r "real" ${LOG_DIR} "committee_online*.log"  | awk '{print $2}' | awk -F[ms] '{total_m += $1; total_s+= $2;count++} END {print total_m/count*60 + total_s/count}'
#
#train_model=$(grep -h -r "End:     train model" ${LOG_DIR} | sed 's|[^0-9]*\([0-9\.]*\)|\1 |g' | awk '{total += $1; count++} END {printf("%f\n",total/count)}')
#upload_data=$(grep -h -r "End:     upload data" ${LOG_DIR} | sed 's|[^0-9]*\([0-9\.]*\)|\1 |g' | awk '{total += $1; count++} END {printf("%f\n",total/count)}')
#
#verify=0
#count=0
#lines=$(grep -h -r "End:     verify the data" ${LOG_DIR})
#while read LINE; do
#	tmp=$(echo $LINE | sed 's|[^0-9]*\([0-9\.]*\)|\1 |g')
#	unit=${LINE: -2}
#	if [ "$unit" = "µs" ]; then
#		verify=$(echo "$verify + $tmp/1000000" | bc -l)
#		#echo "$verify $tmp $unit"
#	elif [ "$unit" = "ms" ]; then
#		verify=$(echo "$verify + $tmp/1000" | bc -l)
#		#echo "$verify $tmp $unit"
#	else
#		verify=$(echo "$verify + $tmp" | bc -l)
#		#echo "$verify $tmp ${unit: -1}"
#	fi
#	count=$((count+1))
#done <<< "$(echo -e "$lines")"
##vunit=$(grep -h -r "End:     verify the data" ${LOG_DIR} | head -1)
#echo "Prover Latency(s)"
#echo "$train_model + $upload_data - 1" | bc
#
#echo "Verifier Latency"
#echo "$verify/$count.0" | bc -l
