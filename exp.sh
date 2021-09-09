cargo build --release
start=$(cat /proc/net/dev | grep "eth0")
in_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $2 }')
out_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $10 }')

sudo  tc qdisc add dev eth0 root netem delay 100ms
sudo tc qdisc change dev eth0 root netem delay 100ms 10ms 25%
for i in {0..3}; do
	#round cts
	time(trickle -s -u 1024 -d 1024 ./target/release/client 2 1 2>&1) | tee client$i.log &
done
wait

end=$(cat /proc/net/dev | grep "eth0")
in_bytes_end=$(echo $end| awk -v OFS=, '/eth0:/ { print $2 }')
out_bytes_end=$(echo $end | awk -v OFS=, '/eth0:/ { print $10 }')
echo "recv bytes " $((in_bytes_end - in_bytes)) >> total.log
echo "sent bytes " $((out_bytes_end - out_bytes)) >> total.log

sudo tc qdisc del dev eth0 root netem
