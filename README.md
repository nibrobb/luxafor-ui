# Luxafor-ui

Goal: create a simple, cross-platform graphical user interface to control the [Luxafor Flag](https://luxafor.com/products/) and perhaps other Luxafor busylights

# Getting started

Get your [prerequisites](https://tauri.app/start/prerequisites/) in order first

## To get this shit working on Windows
```bash
cargo install tauri
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
