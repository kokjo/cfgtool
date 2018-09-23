#!/bin/sh
set -ev

cargo build 
cargo run phase_3
dot -Tpng -o phase_3.png phase_3.dot
