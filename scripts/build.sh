#!/bin/bash -e
# Android SDK without Android Studio
# https://proandroiddev.com/how-to-setup-android-sdk-without-android-studio-6d60d0f2812a
REPO="/home/debian/stackmate-core"
rustup target add x86_64-apple-darwin aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios x86_64-apple-ios

cd $REPO && make android
cargo build

TARGET_DIRECTORY="$REPO/target"
BUILDS_DIRECTORY="$REPO/builds"

rm -rf $BUILDS_DIRECTORY

mkdir -p $BUILDS_DIRECTORY/armv7-linux-androideabi
mkdir -p $BUILDS_DIRECTORY/x86_64-linux-android
mkdir -p $BUILDS_DIRECTORY/aarch64-linux-android
mkdir -p $BUILDS_DIRECTORY/i686-linux-android
mkdir -p $BUILDS_DIRECTORY/x86_64-apple-darwin

cp $TARGET_DIRECTORY/aarch64-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/aarch64-linux-android/
cp $TARGET_DIRECTORY/x86_64-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/x86_64-linux-android/
cp $TARGET_DIRECTORY/i686-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/i686-linux-android/
cp $TARGET_DIRECTORY/armv7-linux-androideabi/release/libstackmate.so $BUILDS_DIRECTORY/armv7-linux-androideabi/

