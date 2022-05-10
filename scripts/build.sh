#!/bin/bash -e

#
# NOTE: THIS SCRIPT MUST BE RUN FROM WITHIN THIS DIRECTORY.
# NOTE: AVOID USING RELATIVE PATH TO CHANGE THIS. USE CARGO MANIFEST DIR ENV VARIABLE?
#

# Android SDK without Android Studio
# https://proandroiddev.com/how-to-setup-android-sdk-without-android-studio-6d60d0f2812a
export LANGUAGE=en_US.UTF-8
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8
export LC_CTYPE=en_US.UTF-8
ANDROID_AARCH64_CLANG=$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/bin/aarch64-linux-android30-clang
ANDROID_ARMV7_CLANG=$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/bin/armv7a-linux-androideabi30-clang
ANDROID_I686_CLANG=$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/bin/i686-linux-android30-clang
ANDROID_X86_64_CLANG=$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/bin/x86_64-linux-android30-clang


export TOOLCHAIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64
export TARGET=aarch64-linux-android
# Set this to your minSdkVersion.
export API=30
# Configure and build.
export AR=$TOOLCHAIN/bin/llvm-ar
export CC=$TOOLCHAIN/bin/$TARGET$API-clang
export AS=$CC
export CXX=$TOOLCHAIN/bin/$TARGET$API-clang++
export LD=$TOOLCHAIN/bin/ld
export RANLIB=$TOOLCHAIN/bin/llvm-ranlib
export STRIP=$TOOLCHAIN/bin/llvm-strip

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
mkdir -p $BUILDS_DIRECTORY/x86_64-apple-darwin

cp $TARGET_DIRECTORY/aarch64-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/aarch64-linux-android/
cp $TARGET_DIRECTORY/x86_64-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/x86_64-linux-android/
cp $TARGET_DIRECTORY/i686-linux-android/release/libstackmate.so $BUILDS_DIRECTORY/i686-linux-android/
cp $TARGET_DIRECTORY/armv7-linux-androideabi/release/libstackmate.so $BUILDS_DIRECTORY/armv7-linux-androideabi/

# strip $BUILDS_DIRECTORY/aarch64-linux-android/libstackmate.so
# strip $BUILDS_DIRECTORY/x86_64-linux-android/libstackmate.so
# strip $BUILDS_DIRECTORY/i686-linux-android/libstackmate.so
# strip $BUILDS_DIRECTORY/armv7-linux-androideabi/libstackmate.so

# zip -r ../builds.zip ../builds
