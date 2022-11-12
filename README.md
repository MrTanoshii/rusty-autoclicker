# Rusty AutoClicker

<div align="center">

<!-- [![Latest Version](https://img.shields.io/badge/Rusty%20AutoClicker-2.2.0-blue)](https://github.com/MrTanoshii/rusty-autoclicker) -->
[![Latest Version](https://img.shields.io/github/v/tag/MrTanoshii/rusty-autoclicker.svg?label=Version&sort=semver&color=orange)](https://github.com/MrTanoshii/rusty-autoclicker/releases)
[![CC0-1.0 License](https://img.shields.io/badge/License-CC0--1.0-blue)](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/LICENSE)
[![Rust Check](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/rust_check.yml)
[![Linux Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/linux_build.yml)
[![macOS Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/macos_build.yml)
[![Windows Build](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml/badge.svg)](https://github.com/MrTanoshii/rusty-autoclicker/actions/workflows/windows_build.yml)

</div>
<div align="center">
  
[![](/screenshots/v2.1.0/rusty-autoclicker_0dnvDPcANp.png?raw=true "Main Interface")](#)
[![](/screenshots/v2.1.0/rusty-autoclicker_a4asg2fXnT.png?raw=true "Hotkey Change")](#)
[![](/screenshots/v2.1.0/rusty-autoclicker_ClJzyc8yHz.png?raw=true "Setting Coordinates")](#)
  
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

# Install `libfontconfig-dev` if you get the following error
# error: failed to run custom build command for `servo-fontconfig-sys v5.1.0`
sudo apt-get install libfontconfig-dev
```

### Running

```shell
rustup update
cargo run --release
```

#### Linux crash fix

```shell
export WINIT_UNIX_BACKEND=x11
./launch_your_app
```

### Build

```shell
rustup update
cargo build --release
```

## Contributing

Please follow the [CONTRIBUTING.md](CONTRIBUTING.md) guide.
