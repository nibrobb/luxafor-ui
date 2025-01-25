# Luxafor-ui

Goal: create a simple, cross-platform graphical user interface to control the [Luxafor Flag](https://luxafor.com/products/) and perhaps other Luxafor busylights

# Getting started

Get your [prerequisites](https://tauri.app/start/prerequisites/) in order first

# Dependencies (Linux only)

## Debian / Ubuntu
```bash
sudo apt install libgtk-3-dev libgdk3.0-cil-dev libatk1.0-dev libxdo-dev\
 librust-gio-sys-dev librust-pango-sys-dev librust-soup3-sys-dev\
 librust-gdk-pixbuf-sys-dev libjavascriptcoregtk-4.1-dev\
 libwebkit2gtk-4.1-dev libayatana-appindicator3-dev # or libappindicator3-dev
```

## NixOS
Use included `shell.nix`


## Common steps
```bash
cargo install --locked --version "^2.0" tauri-cli 
cargo install --locked --no-default-features --features update_check,rustls trunk
rustup target add wasm32-unknown-unknown
```


## Launch this motherfucker with
```bash
cargo tauri dev
```

This template should help get you started developing with Tauri and Leptos.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).


### References
Setting up `trunk` to not use native Open SSL (which is a pain in the ass to set up)
https://users.rust-lang.org/t/install-cargo-trunk-issue-with-x86-64-pc-windows-gnu-target/121119

Luxafor library in rust
https://crates.io/crates/luxafor
