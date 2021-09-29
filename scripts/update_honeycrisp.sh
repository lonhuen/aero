#!/bin/bash
# this is from honeycrisp https://github.com/danxinnoble/honeycrisp
[[ $UID = 0 ]] || exec sudo $0 "$@"

cd
# update the gensetup.sh 
cp /home/ubuntu/quail/lib/honeycrisp/config/genSetupOptions.sh /root/SCALE-MAMBA/genSetupOptions.sh
# update the incr.sh 
cp /home/ubuntu/quail/lib/honeycrisp/test/incr.sh /root/SCALE-MAMBA/incr.sh
# update all the source files
rm -r -f ~/source
mkdir -p ~/source
cp -r /home/ubuntu/quail/lib/honeycrisp/source/* ~/source

cd ~/SCALE-MAMBA

make progs

## make 40 certificates. More can be added as necessary
#mkdir csr
#for ID in {0..39}
#do
#  SUBJ="/CN=player$ID@example.com"
#  openssl genrsa -out Player$ID.key 2048
#  openssl req -new -key Player$ID.key -subj $SUBJ -out csr/Player$ID.csr
#  openssl x509 -req -days 1000 -set_serial 101$ID \
#    -CA RootCA.crt -CAkey RootCA.key \
#    -in csr/Player$ID.csr -out Player$ID.crt 
#done

cd /root/SCALE-MAMBA
for EX in ring ring_test lwe lwe_test decrypt keygen
do
  rm -r -f Programs/$EX
  mkdir Programs/$EX
  cp /root/source/$EX.mpc Programs/$EX/
done 
