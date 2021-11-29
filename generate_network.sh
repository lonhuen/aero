#! /bin/bash
#if [ "$#" -ne 1 ] ; then
#  echo "./run_circuit.sh baseline/online/offline"
#  exit
#fi
#RAYON_NUM_THREADS=6 cargo run --features default,print-trace --release --bin $1 
#
for i in {0..27};do
  str="\"Atom Committee MYVAR\":\n\tdownload: 1000kbps\n\tupload: 1000kbps\n\tdownload-priority: 0\n\tupload-priority: 0\n\tmatch:\n\t- cmdline: .*/committee.* MYVAR"
  echo -e ${str//MYVAR/$i}
done