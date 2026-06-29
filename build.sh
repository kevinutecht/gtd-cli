#!/bin/bash
set -e
cargo build --release
cp target/release/gtd ~/bin/gtd
echo "Installed gtd to ~/bin/gtd"
