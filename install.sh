sudo apt-get update
sudo apt install build-essential
# install rust and cargo
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
cargo build --release
