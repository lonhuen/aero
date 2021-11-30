#! /bin/bash
#if [ "$#" -ne 1 ] ; then
#  echo "./run_circuit.sh baseline/online/offline"
#  exit
#fi
#RAYON_NUM_THREADS=6 cargo run --features default,print-trace --release --bin $1 
#
for i in {0..27};do
  str="\"Atom Committee Offline MYVAR\":\n\tdownload: 1000kbps\n\tupload: 1000kbps\n\tdownload-priority: 0\n\tupload-priority: 0\n\tmatch:\n\t- cmdline: .*/committee_offline MYVAR\$"
  echo -e ${str//MYVAR/$i}
done
for i in {0..27};do
  str="\"Atom Committee Online MYVAR\":\n\tdownload: 1000kbps\n\tupload: 1000kbps\n\tdownload-priority: 0\n\tupload-priority: 0\n\tmatch:\n\t- cmdline: .*/committee_online MYVAR\$"
  echo -e ${str//MYVAR/$i}
done
