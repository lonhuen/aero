APP=$1
echo $APP
cargo build --release

start=$(cat /proc/net/dev | grep "lo")
in_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $2 }')
out_bytes=$(echo $start | awk -v OFS=, '/lo:/ { print $10 }')

./target/release/aggregator_$APP 2>&1 >> aggregator.log &
for i in {0..26}; do 
(./target/release/committee_$APP $i 2>&1 > co$i.log ) &
done
./target/release/committee_$APP 27 | tee co27.log
wait

end=$(cat /proc/net/dev | grep "lo")
in_bytes_end=$(echo $end| awk -v OFS=, '/lo:/ { print $2 }')
out_bytes_end=$(echo $end | awk -v OFS=, '/lo:/ { print $10 }')
echo "recv bytes " $((in_bytes_end - in_bytes))
echo "sent bytes " $((out_bytes_end - out_bytes))
