# CC on Debian

This document provides help on each of the major steps in setting up a fresh debian box as a `cross-compilation environment for stackmate-core android, ios, linux and mac target binaries`.

It is recommended to add all environment variables to `~/.bashrc` to preserve them accross sessions and avoid having to keep initializing them with `export`.


### Setting up debian with basic tools

Debian requires some basic software to get started

```bash
sudo apt-get update --allow-releaseinfo-change
sudo apt-get install -y build-essential cmake apt-transport-https ca-certificates curl gnupg2 software-properties-common dirmngr unzip openssl libssl-dev git expect jq lsb-release tree default-jdk pkg-config
    
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
export PATH="$HOME/.cargo/bin:$PATH"
source $HOME/.cargo/env
echo "[*] Installed basic tools"
```

### Installing android cli-tools

Create a folder called android in your `home`.

```bash
mkdir ~/android
```

Get the command-line only tools from `https://developer.android.com/studio/index.html#command-tools`

As of 20 Apr, 2022

```bash
wget https://dl.google.com/android/repository/commandlinetools-linux-8092744_latest.zip
```

Move the `commandlinetools` into `~/android` and unzip it there.

Example:

```bash
unzip /media/stackmate/android/commandlinetools-linux-8092744_latest.zip 
rm /media/stackmate/android/commandlinetools-linux-8092744_latest.zip
```

This next step is required by the sdkmanager. The cmdline-tools directory needs to have `tools` as its first child. AND all original contents must be in the `tools` directory.

```bash
cd cmdline-tools
mkdir tools
mv -i * tools

# Ignore the error below:
mv: cannot move 'tools' to a subdirectory of itself, 'tools/tools'
```

Verify this path structure:

```
android
└── cmdline-tools
    └── tools
        ├── NOTICE.txt
        ├── bin
        ├── lib
        └── source.properties
```

Create environment variables
```
export ANDROID_HOME=$HOME/android
export PATH=$ANDROID_HOME/cmdline-tools/tools/bin/:$PATH
export PATH=$ANDROID_HOME/platform-tools/:$PATH
```

### Installing android sdk and ndk

```
sdkmanager
```

If you see the following, you're doing well:

```
[==                                     ] 6% Fetch remote repository...
```

IF NOT, stop and trace back your steps.

Too see the sdk and ndk options:

```bash
sdkmanager --list
```

You will mainly need the sdk and ndk for stackmate-core.

```bash
sdkmanager --install "platform-tools" "platforms;android-32" "build-tools;32.0.0" "ndk;21.4.7075529"
```
The NDK contains all the tools required to help us build our rust based C library for android targetted hardware.

**Take note of the installation path and inspect it on the terminal:**

```bash
ls ~/android/sdk/ndk
# The output should show you the  of ndk

# The tools we would need are specifically at the path below
ls /media/stackmate/android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin
export ANDROID_SDK_HOME=/media/stackmate/android
export ANDROID_NDK_HOME=/media/stackmate/android/ndk/21.4.7075529
```

Once you confirm the ndk is at the given path and that the bin folder contains a bunch of binaries, add this path to your `PATH` variable in `.bashrc` for cargo to know where to find 
the binaries for the compiler and linker.

```
export PATH=$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
```

### Pointing cargo to the correct linker binaries

```bash
nano $HOME/.cargo/config
```

Add the following to your global cargo config to point to the correct linker for each build target.

```toml
[target.aarch64-linux-android]
ar = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang"

[target.armv7-linux-androideabi]
ar = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi32-clang"

[target.i686-linux-android]
ar = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android30-clang"

[target.x86_64-linux-android]
ar = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "android/ndk/21.4.7075529/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android30-clang"

[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin14-clang"
ar = "llvm-as"

```

### Building binaries for android targets

```bash
cd 
git clone https://github.com/StackmateNetwork/stackmate-core.git
cd stackmate-core

# add rust targets
rustup target add aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi
make android

```

### Building binaries for macos targets

#### REFER: https://godot-rust.github.io/book/exporting/macosx.html

```bash
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin

sudo apt-get install llvm-dev libclang-dev clang libxml2-dev libz-dev
export MACOSX_CROSS_COMPILER=$HOME/macosx-cross-compiler
install -d $MACOSX_CROSS_COMPILER/osxcross
install -d $MACOSX_CROSS_COMPILER/cross-compiler
cd $MACOSX_CROSS_COMPILER
git clone https://github.com/tpoechtrager/osxcross && cd osxcross

# picked this version as they work well with godot-rust, feel free to change
git checkout 7c090bd8cd4ad28cf332f1d02267630d8f333c19

# move the file where osxcross expects it to be
mv MacOSX10.10.sdk.tar.xz $MACOSX_CROSS_COMPILER/osxcross/tarballs/
# build and install osxcross
UNATTENDED=yes OSX_VERSION_MIN=10.7 TARGET_DIR=$MACOSX_CROSS_COMPILER/cross-compiler ./build.sh


echo "[target.x86_64-apple-darwin]" >> $HOME/.cargo/config
find $MACOSX_CROSS_COMPILER -name x86_64-apple-darwin14-cc -printf 'linker = "%p"\n' >> $HOME/.cargo/config
echo >> $HOME/.cargo/config

```


Add to `~/.cargo/config`

```
[target.x86_64-apple-darwin]
linker = "macosx-cross-compiler/cross-compiler/bin/x86_64-apple-darwin14-cc"
```

```
C_INCLUDE_PATH=$MACOSX_CROSS_COMPILER/cross-compiler/SDK/MacOSX10.10.sdk/usr/include
CC=$MACOSX_CROSS_COMPILER/cross-compiler/bin/x86_64-apple-darwin14-cc
C_INCLUDE_PATH=$MACOSX_CROSS_COMPILER/cross-compiler/SDK/MacOSX10.10.sdk/usr/include CC=$MACOSX_CROSS_COMPILER/cross-compiler/bin/x86_64-apple-darwin14-cc cargo build --release --target x86_64-apple-darwin

```


```
godot --export "Mac OSX" path/to/my.zip

```