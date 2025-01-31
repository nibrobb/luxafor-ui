# Luxafor-ui
>Goal: create a simple, cross-platform graphical user interface to control the [Luxafor Flag](https://luxafor.com/products/) and perhaps other Luxafor busylights

# Getting started

## Installation
Pre-built binaries for the most common operating systems are available from [releases](https://github.com/nibrobb/luxafor-ui/releases), expand 'Assets' then choose the distribution that is right for your system.
If you are on Mac, good luck.

### Post-install

See [POST-INSTALL.md](./POST-INSTALL.md)

## Build it yourself
Get your Tauri [prerequisites](https://tauri.app/start/prerequisites/) in order first

## Dependencies (Debian/Ubuntu only)
```bash
sudo apt install libgtk-3-dev libgdk3.0-cil-dev libatk1.0-dev libxdo-dev\
 librust-gio-sys-dev librust-pango-sys-dev librust-soup3-sys-dev\
 librust-gdk-pixbuf-sys-dev libjavascriptcoregtk-4.1-dev\
 libwebkit2gtk-4.1-dev \
 libayatana-appindicator3-dev # or libappindicator3-dev
```

## NixOS
Use included `shell.nix`

Good luck.


## Common steps
Install the Tauri command line interface `tauri-cli`, the wasm-bundler `trunk` and the wasm32 target

Add the wasm32 build target
```bash
rustup target add wasm32-unknown-unknown
```

If on Apple silicon (M1 or up), install `tauri-cli` and `trunk` from cargo directly.
```bash
cargo install --locked --version "^2.0" tauri-cli
cargo install --locked --no-default-features --features update_check,rustls trunk
```

Pro-tip: Consider using installing `tauri-cli` and `trunk` from [binstall](https://github.com/cargo-bins/cargo-binstall) (not suitable for Apple M1 and up)
```bash
cargo install cargo-binstall
cargo binstall tauri-cli@^2
cargo binstall trunk
```

## Launch the app in development mode
```bash
cargo tauri dev
```

## Build bundles for distribution
```bash
cargo tauri build
```

## Recommended IDE Setup
[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)


### References
#### Setting up `trunk` to not use native Open SSL (which is a pain in the ass to set up)
- https://users.rust-lang.org/t/install-cargo-trunk-issue-with-x86-64-pc-windows-gnu-target/121119

#### Inspiration and udev rules borrowed from
- https://github.com/JnyJny/busylight

#### Luxafor library in rust
- https://crates.io/crates/luxafor

#### Binstall
- https://github.com/cargo-bins/cargo-binstall

#### Built with Tauri and Leptos, bundled with Trunk
- https://tauri.app/
- https://leptos.dev/
- https://trunkrs.dev/

