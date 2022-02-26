#!/bin/bash

cargo clippy
cargo test
# cargo build --release --target aarch64-linux-android
# rustup target add aarch64-linux-android
