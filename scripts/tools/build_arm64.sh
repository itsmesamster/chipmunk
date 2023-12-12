#!/bin/bash
set -eux
uname -a
sudo apt-get install -y libudev-dev
sudo apt-get install ruby-full
sudo apt-get install nodejs
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
gem install dotenv
gem install json
cargo install nj-cli
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
cd ./application/holder
rake release:prod --trace