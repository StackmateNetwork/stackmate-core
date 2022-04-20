#!/bin/bash

#
# NOTE: THIS SCRIPT MUST BE RUN FROM WITHIN THIS DIRECTORY.
# NOTE: AVOID USING RELATIVE PATH TO CHANGE THIS. USE CARGO MANIFEST DIR ENV VARIABLE?
#

# Android SDK without Android Studio
# https://proandroiddev.com/how-to-setup-android-sdk-without-android-studio-6d60d0f2812a

# cargo install cargo-add
# cargo add openssl

# INSTALL ANDROID SDK AND NDK using Android Studio.
 if [[ $(uname) == "Darwin" ]]; then
    # cargo install cargo-lipo
    # cargo install cbindgen
    export ANDROID_NDK_HOME=~/Library/Android/Sdk/ndk/24.0.8215888
    # PHASH=$(echo $admin_password | shasum -a 256)
  else
    export ANDROID_NDK_HOME=~/Android/Sdk/ndk/22.1.7171670
fi
export AARCH64_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android31-clang
export I686_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android31-clang
export X86_64_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android31-clang

echo $AARCH64_LINKER
rm -rf ../builds*
# rm -rf ../target
# cargo clean



mkdir -p ../builds/x86_64-linux-android/release/
mkdir -p ../builds/aarch64-linux-android/release/
mkdir -p ../builds/i686-linux-android/release/
# mkdir -p ../builds/x86_64-unknown-linux-gnu/release/
# mkdir -p ../builds/x86_64-apple-darwin/release/

rustup target add x86_64-apple-darwin aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi
rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios


# [ ! -d osxcross ] && bash $(pwd)/osxcross.sh


# # linux
# assuming that this is run on a linux machine
# cargo build --release

# # ios



# export PATH="$(pwd)/osxcross/target/bin:$PATH"
# export CC=o64-clang
# export CXX=o64-clang++
# export LIBZ_SYS_STATIC=1
# cargo build --release --target x86_64-apple-darwin

# android
 cargo build --release --target aarch64-linux-android
# cargo build --release --target x86_64-linux-android
# cargo build --release --target i686-linux-android

# cd .. && make android

# cp ./target/aarch64-linux-android/release/libstackmate.so ../builds/aarch64-linux-android/release/
# cp ./target/x86_64-linux-android/release/libstackmate.so ../builds/x86_64-linux-android/release/
# cp ./target/i686-linux-android/release/libstackmate.so ../builds/i686-linux-android/release/
# cp ./target/armv7-linux-androideabi/release/libstackmate.so ../builds/armv7-linux-androideabi/release/
# strip ../builds/aarch64-linux-android/release/libstackmate.so
# strip ../builds/x86_64-linux-android/release/libstackmate.so
# strip ../builds/i686-linux-android/release/libstackmate.so

# zip -r ../builds.zip ../builds
