# CC on Debian

This documment provides help on each of the major steps in setting up a fresh debian box as a cross-compilation environment for stackmate-core android target binaries

### Setting up debian with basic tools
Debian requires some basic software to get started

```bash

sudo apt-get update --allow-releaseinfo-change
sudo apt-get install -y \
    build-essential \
    cmake \ 
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg2 \
    software-properties-common \
    dirmngr \
    unzip \
    git \
    expect \
    jq \
    lsb-release 
    
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
export PATH="$HOME/.cargo/bin:$PATH"
source $HOME/.cargo/env
cargo install pier
echo "[*] Installed basic tools"
```

### Installing android cli-tools

Get the command-line only tools from `https://developer.android.com/studio/index.html#command-tools`
    
As of 20 Apr, 2022
```bash
wget https://dl.google.com/android/repository/commandlinetools-linux-8092744_latest.zip
```

Create a folder called android in your `home`.
```bash
mkdir ~/android
```

This will be and env `$ANDROID_HOME`

Move the `commandlinetools` into `~/android` and unzip it there.


Example:
```bash
mv ~/commandlinetools-linux-8092744_latest.zip ~/android
unzip ~/android/commandlinetools-linux-8092744_latest.zip 
rm ~/android/commandlinetools-linux-8092744_latest.zip
```

This next step is required by the sdkmanager. The cmdline-tools directory needs to have `tools` as its first child. AND all original contents must be in the `tools` directory.

```bash
$ cd cmdline-tools
$ mkdir tools
$ mv -i * tools
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

Add the following to your `~/.bashrc`

```
export ANDROID_HOME=$HOME/android
export PATH=$ANDROID_HOME/cmdline-tools/tools/bin/:$PATH
export PATH=$ANDROID_HOME/emulator/:$PATH
export PATH=$ANDROID_HOME/platform-tools/:$PATH
```

### Installing android sdk and ndk

```
sdkmanager
```

If you see the following, youre doing well:

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
sdkmanager --install "platform-tools" "platforms;android-32" "build-tools;32.0.0" "emulator"
```

The NDK contains all the tools required to help us build our rust based C library for android targetted hardware.


<b>Take note of the installation path and inspect it on the terminal:</b>

```
ls ~/android/sdk/ndk
# The output should show you the <version_number> of ndk

# The tools we would need are specifically at the path below
ls ~/android/sdk/ndk/<version_number>/toolchains/llvm/prebuilt/linux-x86_64/bin

```
Once you confirm the ndk is at the given path and that the bin folder contains a bunch of binaries, add this path to your `PATH` variable in `.bashrc` for cargo to know where to find the binaries for the compiler and linker.

```
export PATH=$PATH:$HOME/android/sdk/ndk/<version_number>/toolchains/llvm/prebuilt/linux-x86_64/bin
```

It is safe to explicitly specify which specific linker to use per build target. 

Add the following to your global cargo config @ `$HOME/.cargo/config` to point to the correct linker for each build target.

Make sure to substitute with the appropriate <version_number>

```toml
[target.aarch64-linux-android]
ar = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-as"
linker = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android31-clang"

[target.armv7-linux-androideabi]
ar = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-as"
linker = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi31-clang"

[target.i686-linux-android]
ar = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-as"
linker = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android31-clang"

[target.x86_64-linux-android]
ar = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-as"
linker = "~/Library/Android/sdk/ndk/24.0.8215888/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android31-clang"

[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin14-clang"
ar = "llvm-as"

```

### Building binaries for android targets