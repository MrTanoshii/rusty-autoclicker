# Rusty AutoClicker

<div align="center">
  
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v1.0.0/rusty-autoclicker_15o7LSwR3T.png)](#)
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v1.0.0/rusty-autoclicker_ucpN0Ra6EU.png)](#)
  
</div>

## Building from source

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel`

For running the `build_web.sh` script you also need to install `jq` and `binaryen` with your packet manager of choice.
