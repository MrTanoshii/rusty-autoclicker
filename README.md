# Rusty AutoClicker

<div align="center">

[![Latest Version](https://img.shields.io/badge/Rusty%20AutoClicker-2.0.0-blue)](https://github.com/MrTanoshii/rusty-autoclicker)
[![CC0-1.0 License](https://img.shields.io/badge/License-CC0--1.0-blue)](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/LICENSE)
[![Rust Check](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml)
[![Linux Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml)
[![macOS Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml)
[![Windows Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml)

</div>
<div align="center">
  
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v2.0.0/rusty-autoclicker_ViGggxUHWg)](#)
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v2.0.0/rusty-autoclicker_ULHtfvIyAM)](#)
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v2.0.0/rusty-autoclicker_qcROvJWUlY)](#)
  
</div>

## Building from source

### OS specific requirements

#### Fedora Rawhide (not tested)

```shell
dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel
```

#### Linux

```shell
sudo apt-get install libx11-dev libxtst-dev libevdev-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev
```

### Running

```shell
rustup update
cargo run --release
```

### Build

```shell
rustup update
cargo build --release
```
