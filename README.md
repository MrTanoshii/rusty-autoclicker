# Rusty AutoClicker
<div align="center">
  
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v1.0.0/rusty-autoclicker_15o7LSwR3T.png)](#)
[![](https://github.com/MrTanoshii/rusty-autoclicker/blob/main/screenshots/v1.0.0/rusty-autoclicker_ucpN0Ra6EU.png)](#)
  
</div>

### Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel`

For running the `build_web.sh` script you also need to install `jq` and `binaryen` with your packet manager of choice.

### Compiling for the web

Make sure you are using the latest version of stable rust by running `rustup update`.

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page. For this you need to set up some tools. There are a few simple scripts that help you with this:

```sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```

- `setup_web.sh` installs the tools required to build for web
- `build_web.sh` compiles your code to wasm and puts it in the `docs/` folder (see below)
- `start_server.sh` starts a local HTTP server so you can test before you publish
- Open http://127.0.0.1:8080/ in a web browser to view

The finished web app is found in the `docs/` folder (this is so that you can easily share it with [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)). It consists of three files:

- `index.html`: A few lines of HTML, CSS and JS that loads your app. **You need to edit this** (once) to replace `eframe_template` with the name of your crate!
- `your_crate_bg.wasm`: What the Rust code compiles to.
- `your_crate.js`: Auto-generated binding between Rust and JS.

You can test the template app at <https://emilk.github.io/eframe_template/>.

## Updating egui

As of 2022, egui is in active development with frequent releases with breaking changes. [eframe_template](https://github.com/emilk/eframe_template/) will be updated in lock-step to always use the latest version of egui.

When updating `egui` and `eframe` it is recommended you do so one version at the time, and read about the changes in [the egui changelog](https://github.com/emilk/egui/blob/master/CHANGELOG.md) and [eframe changelog](https://github.com/emilk/egui/blob/master/eframe/CHANGELOG.md).
