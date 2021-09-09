sudo apt-get update
sudo apt install build-essential
sudo apt install trickle
# install rust and cargo
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
cargo build --release
git config --global core.editor "vim"
