#!/bin/sh
cargo build --release
killall bansheeniri
cp ./target/release/bansheeniri $HOME/.local/bin/bansheeniri
