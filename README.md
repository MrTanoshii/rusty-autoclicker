# Rusty AutoClicker

<div align="center">

[![Latest Version](https://img.shields.io/badge/Rusty%20AutoClicker-1.1.0-blue)](https://github.com/MrTanoshii/rusty-autoclicker)
[![CC0-1.0 License](https://img.shields.io/badge/License-CC0--1.0-blue)](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/LICENSE)
[![Rust Check Status](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml)
[![Linux Build Status](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml)
[![macOS Build Status](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml)
[![Windows Build Status](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml)

</div>
<div align="center">
  
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v1.0.0/rusty-autoclicker_15o7LSwR3T.png)](#)
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v1.0.0/rusty-autoclicker_ucpN0Ra6EU.png)](#)
  
</div>

## Building from source

### OS specific requirements

#### Fedora Rawhide (not tested)

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel`

#### Linux

`sudo apt-get install libx11-dev libxtst-dev libevdev-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

### Running

```shell
rustup update
cargo run --release
```
