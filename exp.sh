cargo build --release
start=$(cat /proc/net/dev | grep "eth0")
in_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $2 }')
out_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $10 }')

#sudo tt eth0 network.yaml &

for i in {0..2}; do
	time(./target/release/client $i 2>&1 > client$i.log) &
done
time(./target/release/client $((i+1)) 2>&1) | tee client$((i+1)).log
wait

end=$(cat /proc/net/dev | grep "eth0")
in_bytes_end=$(echo $end| awk -v OFS=, '/eth0:/ { print $2 }')
out_bytes_end=$(echo $end | awk -v OFS=, '/eth0:/ { print $10 }')
echo "recv bytes " $((in_bytes_end - in_bytes)) > total.log
echo "sent bytes " $((out_bytes_end - out_bytes)) >> total.log
