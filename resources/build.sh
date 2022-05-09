#!/bin/bash -e

#
# NOTE: THIS SCRIPT MUST BE RUN FROM WITHIN THIS DIRECTORY.
# NOTE: AVOID USING RELATIVE PATH TO CHANGE THIS. USE CARGO MANIFEST DIR ENV VARIABLE?
#

# Android SDK without Android Studio
# https://proandroiddev.com/how-to-setup-android-sdk-without-android-studio-6d60d0f2812a
REPO="/home/debian/stackmate-core"
rustup target add x86_64-apple-darwin aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios x86_64-apple-ios

# android
# cargo build --release --target aarch64-linux-android
# cargo build --release --target x86_64-linux-android
# cargo build --release --target i686-linux-android

cd .. && make android
cargo build

# rm -rf target/release

TARGET_DIRECTORY="$REPO/target"
BUILDS_DIRECTORY="$REPO/builds"

rm -rf $BUILDS_DIRECTORY

mkdir -p $BUILDS_DIRECTORY/armv7-linux-androideabi
mkdir -p $BUILDS_DIRECTORY/x86_64-linux-android
mkdir -p $BUILDS_DIRECTORY/aarch64-linux-android
mkdir -p $BUILDS_DIRECTORY/i686-linux-android

cp $TARGET_DIRECTORY/aarch64-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/aarch64-linux-android/
cp $TARGET_DIRECTORY/x86_64-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/x86_64-linux-android/
cp $TARGET_DIRECTORY/i686-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/i686-linux-android/
cp $TARGET_DIRECTORY/armv7-linux-androideabi/release/libstackmate.so $BUILDS_DIRECTORY/armv7-linux-androideabi/

# strip $BUILDS_DIRECTORY/aarch64-linux-android/libstackmate.so
# strip $BUILDS_DIRECTORY/x86_64-linux-android/libstackmate.so
# strip $BUILDS_DIRECTORY/i686-linux-android/libstackmate.so
# strip $BUILDS_DIRECTORY/armv7-linux-androideabi/libstackmate.so

# zip -r ../builds.zip ../builds
