#!/bin/bash

cargo build --release

# Example command
./target/release/humanizer_rs -k testllc,admin,cloud,service -o output.txt