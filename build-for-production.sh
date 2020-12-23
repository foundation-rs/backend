#!/bin/bash

cargo build --release
strip target/release/server
cp target/release/server distribution/bin/