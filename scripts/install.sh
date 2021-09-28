sudo apt-get update
sudo apt install build-essential -y
#sudo apt install trickle -y
# install rust and cargo
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
cargo build --release
git config --global core.editor "vim"
sudo apt install python3-pip -y
sudo pip3 install traffictoll -y
