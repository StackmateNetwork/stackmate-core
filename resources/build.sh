#!/bin/bash

rm -rf ../builds*
# rm -rf ../target
# cargo clean

# AARCH64_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android29-clang
# I686_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android29-clang
# X86_64_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android29-clang

mkdir -p ../builds/x86_64-linux-android/release/
mkdir -p ../builds/aarch64-linux-android/release/
# mkdir -p ../builds/i686-linux-android/release/
# mkdir -p ../builds/x86_64-unknown-linux-gnu/release/
# mkdir -p ../builds/x86_64-apple-darwin/release/

# rustup target add x86_64-apple-darwin
rustup target add aarch64-linux-android
# rustup target add i686-linux-android
rustup target add x86_64-linux-android

# [ ! -d osxcross ] && bash $(pwd)/osxcross.sh


# # linux
# # osx
# export PATH="$(pwd)/osxcross/target/bin:$PATH"
# export CC=o64-clang
# export CXX=o64-clang++
# export LIBZ_SYS_STATIC=1
# cargo build --release --target x86_64-apple-darwin
# android

# cargo build --release
cargo build --release --target aarch64-linux-android
cargo build --release --target x86_64-linux-android
# cargo build --release --target i686-linux-android

cp ../target/x86_64-linux-android/release/libstackmate.a ../builds/x86_64-linux-android/release/
cp ../target/aarch64-linux-android/release/libstackmate.d ../builds/aarch64-linux-android/release/
# cp ../target/i686-linux-android/release/libstackmate.so ../builds/i686-linux-android/release/

zip -r ../builds.zip ../builds
