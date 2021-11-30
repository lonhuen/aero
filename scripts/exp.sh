#! /bin/bash
app=$1
nr_real=$2

cd ~/quail

if [ -z $nr_real ]; then
  nr_real=$(grep "^nr_real[:space]=" ./config.yaml | awk -F[=] '{print $2}')
fi

#cargo build --release
#start=$(cat /proc/net/dev | grep "eth0")
#in_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $2 }')
#out_bytes=$(echo $start | awk -v OFS=, '/eth0:/ { print $10 }')
start=$(cat /proc/net/dev | grep "ens5")
in_bytes=$(echo $start | awk -v OFS=, '/ens5:/ { print $2 }')
out_bytes=$(echo $start | awk -v OFS=, '/ens5:/ { print $10 }')

for i in `seq 0 $((nr_real-2))`; do
	(time RAYON_NUM_THREADS=6 ./$app/target/release/client $i) &> client$i.log &
done
(time RAYON_NUM_THREADS=6 ./$app/target/release/client $((i+1)))  &> client$((i+1)).log
wait

#end=$(cat /proc/net/dev | grep "eth0")
#in_bytes_end=$(echo $end| awk -v OFS=, '/eth0:/ { print $2 }')
#out_bytes_end=$(echo $end | awk -v OFS=, '/eth0:/ { print $10 }')
end=$(cat /proc/net/dev | grep "ens5")
in_bytes_end=$(echo $end| awk -v OFS=, '/ens5:/ { print $2 }')
out_bytes_end=$(echo $end | awk -v OFS=, '/ens5:/ { print $10 }')
echo "recv bytes " $((in_bytes_end - in_bytes)) > total.log
echo "sent bytes " $((out_bytes_end - out_bytes)) >> total.log
