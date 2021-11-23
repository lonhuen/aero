#! /bin/bash
nr_real=$1

if [ -z $nr_real ]; then
  nr_real=$(grep "^nr_real[:space]=" ./config.yaml | awk -F[=] '{print $2}')
fi

for i in `seq 0 $((nr_real-2))`; do
	RAYON_NUM_THREADS=6 ./target/release/client $i 2>&1 > client$i.log &
done
RAYON_NUM_THREADS=6 ./target/release/client $((i+1)) 2>&1 > client$((i+1)).log
wait
