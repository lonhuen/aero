#! /bin/bash
APP=$1

start=$(cat /proc/net/dev | grep "ens5")
in_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $2 }')
out_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $10 }')

# ./atom/target/release/aggregator_$APP 2>&1 > aggregator.log &
for i in {0..26}; do 
	(./atom/target/release/committee_$APP $i 2>&1 > committee_${APP}$i.log ) &
done
time(./atom/target/release/committee_$APP 27) | tee comittee_${APP}$((i+1)).log
wait

end=$(cat /proc/net/dev | grep "ens5")
in_bytes_end=$(echo $end| awk -v OFS=, '/lo:/ { print $2 }')
out_bytes_end=$(echo $end | awk -v OFS=, '/lo:/ { print $10 }')
echo "recv bytes " $((in_bytes_end - in_bytes))
echo "sent bytes " $((out_bytes_end - out_bytes))