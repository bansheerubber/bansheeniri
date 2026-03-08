#!/bin/sh
cargo build --release
killall bansheeniri
mkdir -p ~/.local/share/bansheeniri
cp ./target/release/bansheeniri $HOME/.local/bin/bansheeniri
