#! /bin/bash
APP=$1

start=$(cat /proc/net/dev | grep "lo")
in_lo_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $2 }')
out_lo_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $10 }')
#echo $start
#start=$(cat /proc/net/dev | grep "ens5")
start=$(cat /proc/net/dev | grep "ens5")
in_ens5_bytes=$(echo $start | awk -v OFS=, '/ens5:/ { print $2 }')
out_ens5_bytes=$(echo $start | awk -v OFS=, '/ens5:/ { print $10 }')
in_bytes=$((in_lo_bytes + in_ens5_bytes))
out_bytes=$((out_lo_bytes + out_ens5_bytes))
#echo $start
#echo $in_bytes
#echo $out_bytes

## ./atom/target/release/aggregator_$APP 2>&1 > aggregator.ens5g &
for i in {0..26}; do 
	(./atom/target/release/committee_$APP $i 2>&1 > committee_${APP}$i.log ) &
done
(time ./atom/target/release/committee_$APP 27) 2>&1 | tee committee_${APP}$((i+1)).log
wait

end=$(cat /proc/net/dev | grep "lo")
in_lo_bytes_end=$(echo $end|  grep "lo" | awk -v OFS=, '/lo:/ { print $2 }')
out_lo_bytes_end=$(echo $end | grep "lo" | awk -v OFS=, '/lo:/ { print $10 }')
end=$(cat /proc/net/dev | grep "ens5")
in_ens5_bytes_end=$(echo $end|  grep "ens5" | awk -v OFS=, '/ens5:/ { print $2 }')
out_ens5_bytes_end=$(echo $end | grep "ens5" | awk -v OFS=, '/ens5:/ { print $10 }')
in_bytes_end=$((in_lo_bytes_end + in_ens5_bytes_end))
out_bytes_end=$((out_lo_bytes_end + out_ens5_bytes_end))
echo "recv bytes " $((in_bytes_end - in_bytes)) | tee -a committee_${APP}$((i+1)).log
echo "sent bytes " $((out_bytes_end - out_bytes)) | tee -a committee_${APP}$((i+1)).log 
