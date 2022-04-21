#!/bin/bash

#
# NOTE: THIS SCRIPT MUST BE RUN FROM WITHIN THIS DIRECTORY.
# NOTE: AVOID USING RELATIVE PATH TO CHANGE THIS. USE CARGO MANIFEST DIR ENV VARIABLE?
#

# Android SDK without Android Studio
# https://proandroiddev.com/how-to-setup-android-sdk-without-android-studio-6d60d0f2812a

rm -rf ../builds*

mkdir -p ../builds/x86_64-linux-android/release/
mkdir -p ../builds/aarch64-linux-android/release/
mkdir -p ../builds/i686-linux-android/release/

rustup target add x86_64-apple-darwin aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios

# android
 cargo build --release --target aarch64-linux-android
# cargo build --release --target x86_64-linux-android
cargo build --release --target i686-linux-android

# cd .. && make android

# cp ./target/aarch64-linux-android/release/libstackmate.so ../builds/aarch64-linux-android/release/
# cp ./target/x86_64-linux-android/release/libstackmate.so ../builds/x86_64-linux-android/release/
# cp ./target/i686-linux-android/release/libstackmate.so ../builds/i686-linux-android/release/
# cp ./target/armv7-linux-androideabi/release/libstackmate.so ../builds/armv7-linux-androideabi/release/
# strip ../builds/aarch64-linux-android/release/libstackmate.so
# strip ../builds/x86_64-linux-android/release/libstackmate.so
# strip ../builds/i686-linux-android/release/libstackmate.so

# zip -r ../builds.zip ../builds
