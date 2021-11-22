if [ "$#" -ne 1 ] ; then
  echo "./run_circuit.sh baseline/online/offline"
  exit
fi
RAYON_NUM_THREADS=6 cargo run --release --bin $1 
