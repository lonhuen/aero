dimension=(4096)
poly=(1)
#bench=("baseline" "offline" "online")
bench=("baseline")
if [ "$#" -ne 1 ] ; then
  for b in "${bench[@]}"; do
    for d in "${dimension[@]}"; do
      for p in "${poly[@]}"; do
        #RAYON_NUM_THREADS=1 cargo run --release --example $b $d $p log/polymr${d}_${p}.log
        cargo run --release --example $b $d $p log/polymr${d}_${p}.log | tee -a test.log
      done
    done
  done
else
    for d in "${dimension[@]}"; do
      for p in "${poly[@]}"; do
        #RAYON_NUM_THREADS=1 cargo run --release --example $b $d $p log/polymr${d}_${p}.log
        #cargo run --release --example $1 $d $p log/polymr${d}_${p}.log | tee -a test.log
        #RAYON_NUM_THREADS=1 cargo run --features "print-trace" --release --example $1 $d $p log/polymr${d}_${p}.log | tee -a test.log
        cargo run --features "print-trace" --release --example $1 $d $p log/polymr${d}_${p}.log | tee -a test.log
        #cargo run --release --example $1 $d $p log/polymr${d}_${p}.log | tee -a test.log
      done
    done
fi
