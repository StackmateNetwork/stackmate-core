# Rust-C-Dart FFI

The goal of this guide is to provide a process for using Rust code in Dart projects for Android and iOS build targets via C ffi.

## Part 1: FFI Intro & Build targets for Android

This part covers 

- how to create C bindings for a custom rust lib
- compiling the lib for use on Android targets

This example is only tested on linux but the logic is cross platform

### Tooling

- cargo 
- Android ndk

### Install & Setup

#### cargo
```bash
# Install rust toolchain
curl https://sh.rustup.rs -sSf | sh
cargo --version
```
#### Android ndk

The NDK contains all the tools required to help us build our rust based C library for android targetted hardware.

The quickest way to get the ndk is via [Android Studio](https://developer.android.com/studio).

Once Android studio is installed, setup a new project to get into the IDE. 

Find the SDK Manager as an icon on the top right of the screen, OR navigate to `Android Studio > Preferences > Appearance & Behaviour > Android SDK > SDK Tools`.

Here you can chose to install the Android NDK.

<b>Take note of the installation path and inspect it on the terminal:</b>

```
ls $HOME/Android/Sdk/ndk
# The output should show you the <version_number> of ndk

# The tools we would need are specifically at the path below
ls $HOME/Android/Sdk/ndk/<version_number>/toolchains/llvm/prebuilt/linux-x86_64/bin
# macosx 
~/Library/Android/Sdk/ndk/<version_number>/toolchains/llvm/prebuilt/darwin-x86_64/bin

```

Once you confirm the ndk is at the given path and that the bin folder contains a bunch of binaries, add this path to your `PATH` variable for cargo.

```
export PATH=$PATH:$HOME/Android/Sdk/ndk/<version_number>/toolchains/llvm/prebuilt/linux-x86_64/bin
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

## ON MAC
Add the following to .zshrc
```
ANDROID_SDK="/path/to/sdk"
ANDROID_NDK="/path/to/ndk"
ANDROID_TOOLCHAIN="/path/to/toolchain"
PATH="$PATH:$ANDROID_TOOLCHAIN/bin"
export CC="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin/clang -target armv7-none-linux-androideabi -gcc-toolchain $NDK_TOOLCHAIN"
export CXX="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin/clang++ -target armv7-none-linux-androideabi -gcc-toolchain $NDK_TOOLCHAIN"
```

PROBABLY NOT NEEDED
```
export HOMEBREW_NO_ANALYTICS=1

brew update
brew upgrade
brew install gcc
brew cleanup
```

```
brew install nspr ant
```

Finally, add toolchains for our build targets

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```
> NOTE:

The linker binary for all targets are named slightly differently from their target name, eg: target: `aarch64-linux-android` has linker:`aarch64-linux-android29-clang`, this can change based on the version you have. Inspect your bin to see which versions are available and pick accordingly per linker.


### Write Rust Function With C Binding

```bash
cargo new --lib stackmate-core && cd stackmate-core
nano Cargo.toml
```
```toml
[package]
name = "stackmate-core"
version = "0.1.0"
edition = "2018"

[lib]
name = "stackmate-core"
crate-type = ["staticlib", "cdylib"]

[dependencies]
rand = "0.6.0"
bip39 = "1.0.1"

```

```bash
nano src/lib.rs
```

Binding a rust function to a C function requires translating input and output types to and from native types. 

For simlicity, it is best to start with only working with strings, specifically the `CString` type.

Considering the conversions to and from CStrings as boiler plate; you can use any snippet of rust code and translate it to a C compatible function.

It makes sense to create your own custom use cases of specific libraries by just wrapping its usage into a purely `CString` interface.

We are using rand and bip39 to create a menmonic bitcoin seed phrase.

Notice that just as the function `mnemonic` converts the `length` input CString input into a native rust type `len`, in `test_mnemonic`,the output `mnemonic_ptr` of the function `mnemonic` is converted into a native rust type `mnemonic_native`. When working with `CString` on the input side, we use an `unsafe` block to extract the value from a pointer* which could potentially be a null and break rust rules.

Once you get around the verbosity of it, its not all that intense.


```rust
use std::os::raw::{c_char};
use std::ffi::{CString,CStr};

use rand::rngs::OsRng;
use bip39::{Language, Mnemonic};

#[no_mangle]
pub extern fn mnemonic(length: *const c_char) -> *mut c_char {
    // convert from CString inputs
    let input_cstr = unsafe {CStr::from_ptr(length)};
    let len:usize = match input_cstr.to_str(){
        Err(_) => 12,
        Ok(string)=> string.parse::<usize>().unwrap()
    };
    // regular rust code
    let mut rng = OsRng::new().expect("!!OsRng Error!!");
    let mnemonic = Mnemonic::generate_in_with(&mut rng, Language::English, len)
        .unwrap()
        .to_string();

    // convert to CString outputs
    CString::new(mnemonic).unwrap().into_raw()

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic() {
        let length = 12;
        let length_cstr = CString::new(length.to_string()).unwrap().into_raw();
        let mnemonic_ptr = mnemonic(length_cstr);
        println!("Output C *char: {:?}",mnemonic_ptr);
        let mnemonic_cstr = unsafe {CStr::from_ptr(mnemonic_ptr)};
        let mnemonic_native = mnemonic_cstr.to_str().unwrap();
        println!("Output Rust &str: {}",mnemonic_native);
    }


}
```

### Test

```bash
cargo test -- --nocapture
# You will see the mnemonic generated in the output as both a C pointer and Rust native &str
```
```text
...
running 1 test
Output C *char: 0x7f86bc000cf0
Output Rust &str: blade rose lift later expand math story broccoli damp cruel lava video
test tests::test_mnemonic ... ok

```

### Build for Android Target

cargo can compile binaries for specific target hardware. Check the list of [Supported Platforms](https://doc.rust-lang.org/nightly/rustc/platform-support.html). Our targets are all under Tier 2.


```bash
cd path/to/project
cargo clean
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release
cargo build --target i686-linux-android --release
cargo build --target x86_64-linux-android --release

```

## ON Mac

```
ANDROID_HOME=$ANDROID_SDK NDK_HOME=$ANDROID_NDK NDK_STANDALONE=$ANDROID_TOOLCHAIN cargo build --target aarch64-linux-android --release

```

## Debugging tips

Incase you bump into errors along the way, before rebuilding clean up any artifacts from previous builds.

```bash
rm -rf stackmate-core/target
cargo clean
```

Check if you have the correct linker binaries by running them manually:

```bash
/media/stackmate/android/ndk/24.0.8215888/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android32-clang --version
```

## Part 2: Using Android builds with Dart-C FFI

In the next part, we will call this C library via Dart on an Android device. 

## Part 3: Build Targets for iOS

Then we will build the same library for iOS target

## Part 4: Using iOS builds with Dart-C FFI

Then we will call this C library via Dart on an iOS device.

## Part 5: Automating for easier development

Finally, we will create a script to facilitate continous development.