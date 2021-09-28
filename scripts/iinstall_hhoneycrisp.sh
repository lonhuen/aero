sudo su
apt-get update && apt-get install -y \
  bzip2 \
  doxygen \
  g++ \
  gcc \
  git \
  libgmp3-dev \
  m4 \
  make \
  patch \
  python \
  tmux \
  vim \
  wget \
  yasm
cd 
#frigate install supporting mpir library
wget http://mpir.org/mpir-3.0.0.tar.bz2
tar -xvf mpir-3.0.0.tar.bz2
rm mpir-3.0.0.tar.bz2
cd mpir-3.0.0
./configure --enable-cxx
make install
cp .libs/* /usr/lib64/

# install openssl from source because apparently apt's version is not good
# enough
cd
wget https://www.openssl.org/source/openssl-1.1.0h.tar.gz
tar -xvf openssl-1.1.0h.tar.gz
rm openssl-1.1.0h.tar.gz
cd openssl-1.1.0h
./config
make
make install
ldconfig 

# download SCALE-MAMBA
cd
git clone https://github.com/KULeuven-COSIC/SCALE-MAMBA.git
cd SCALE-MAMBA
git checkout -b v1.2 3722a85
cp /home/ubuntu/quail/lib/honeycrisp/config/CONFIG.mine .

# Custom IO
cp /home/ubuntu/quail/lib/honeycrisp/source/Player.cpp ./src/
cp /home/ubuntu/quail/lib/honeycrisp/source/IO.h ./src/Input_Output/
cp /home/ubuntu/quail/lib/honeycrisp/source/Input_Output_File.cpp ./src/Input_Output/
cp /home/ubuntu/quail/lib/honeycrisp/source/Input_Output_File.h ./src/Input_Output/

cp /home/ubuntu/quail/lib/honeycrisp/config/config.h ./src/config.h

# Allow for custom Shamir evaluation points
cp /home/ubuntu/quail/lib/honeycrisp/SMtweaks/MSP.cpp ./src/LSSS/
cp /home/ubuntu/quail/lib/honeycrisp/SMtweaks/MSP.h ./src/LSSS/
cp /home/ubuntu/quail/lib/honeycrisp/SMtweaks/ShareData.cpp ./src/LSSS/
cp /home/ubuntu/quail/lib/honeycrisp/SMtweaks/ShareData.h ./src/LSSS/
cp /home/ubuntu/quail/lib/honeycrisp/SMtweaks/Setup.cpp ./src/

cp /home/ubuntu/quail/lib/honeycrisp/source/benchmark.sh .

# Scripts to allow changes to sharing scheme
cp /home/ubuntu/quail/lib/honeycrisp/config/publicin.txt .
cp /home/ubuntu/quail/lib/honeycrisp/config/genSetupOptions.sh .
cp /home/ubuntu/quail/lib/honeycrisp/config/chooseSubset.py .
cp /home/ubuntu/quail/lib/honeycrisp/config/modifyEvalPoints.sh .
cp /home/ubuntu/quail/lib/honeycrisp/config/renameShares.sh .
cp /home/ubuntu/quail/lib/honeycrisp/test/testReconstruct.sh .
#cp /home/ubuntu/quail/lib/honeycrisp/test/test0.sh .
#cp /home/ubuntu/quail/lib/honeycrisp/test/testd.sh .
cp /home/ubuntu/quail/lib/honeycrisp/test/incr.sh .

make progs

# set up certificate authority
SUBJ="/CN=www.example.com"
cd Cert-Store

openssl genrsa -out RootCA.key 4096
openssl req -new -x509 -days 1826 -key RootCA.key \
           -subj $SUBJ -out RootCA.crt

# make 40 certificates. More can be added as necessary
mkdir csr
for ID in {0..39}
do
  SUBJ="/CN=player$ID@example.com"
  openssl genrsa -out Player$ID.key 2048
  openssl req -new -key Player$ID.key -subj $SUBJ -out csr/Player$ID.csr
  openssl x509 -req -days 1000 -set_serial 101$ID \
    -CA RootCA.crt -CAkey RootCA.key \
    -in csr/Player$ID.csr -out Player$ID.crt 
done

# Set up SCALE-MAMBA
cd ~/SCALE-MAMBA
./genSetupOptions.sh 4 1 | ./Setup.x  # By default set-up with 4 players

# copy examples to correct locations
cd ~/SCALE-MAMBA
for EX in ring ring_test lwe lwe_test decrypt keygen
do
  mkdir Programs/$EX
  cp /home/ubuntu/quail/lib/honeycrisp/source/$EX.mpc Programs/$EX/
done 

for EX in input_shares output_shares
do
  mkdir Programs/$EX
  cp /home/ubuntu/quail/lib/honeycrisp/test/$EX.mpc Programs/$EX
done

cd /home/ubuntu/quail/lib/honeycrisp/config
for x in chooseSubset.py renameShare.sh genSetupMSP.sh
do 
	mkdir config/
	cp $x config/$x
done

# add simple syntax highlighting
cd
mkdir -p .vim/syntax
cp config/mamba.vim .vim/syntax
mkdir .vim/ftdetect
cd .vim/ftdetect
echo "au BufNewFile,BufRead *.wir set filetype=mamba" > mamba.vim

# Compile necessary files
cd ~/SCALE-MAMBA
