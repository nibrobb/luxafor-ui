# Luxafor-ui

<p align="center">
    <img src="./public/gui.png" alt="Luxafor-ui in dark mode" />
</p>
<br clear="right"/>

# Getting started

## Installation
Go to [Releases](https://github.com/nibrobb/luxafor-ui/releases), expand 'Assets', then choose the distribution that is right for your system.
If you are on Mac, good luck.

## Slack integration
Control your busylight directly from Slack!

> [!NOTE]
> Luxafor-ui must be running when adding to slack since the local app stores the users access tokens

<a href="https://slack.com/oauth/v2/authorize?scope=users%3Aread&amp;user_scope=users.profile%3Aread%2Cusers.profile%3Awrite&amp;redirect_uri=https%3A%2F%2Flocalhost%3A8080%2Fauth%2Finstall&amp;client_id=7816636666498.7805940374471" style="align-items:center;color:#fff;background-color:#4A154B;border:0;border-radius:48px;display:inline-flex;font-family:Lato, sans-serif;font-size:16px;font-weight:600;height:48px;justify-content:center;text-decoration:none;width:236px"><svg xmlns="http://www.w3.org/2000/svg" style="height:20px;width:20px;margin-right:12px" viewBox="0 0 122.8 122.8"><path d="M25.8 77.6c0 7.1-5.8 12.9-12.9 12.9S0 84.7 0 77.6s5.8-12.9 12.9-12.9h12.9v12.9zm6.5 0c0-7.1 5.8-12.9 12.9-12.9s12.9 5.8 12.9 12.9v32.3c0 7.1-5.8 12.9-12.9 12.9s-12.9-5.8-12.9-12.9V77.6z" fill="#e01e5a"></path><path d="M45.2 25.8c-7.1 0-12.9-5.8-12.9-12.9S38.1 0 45.2 0s12.9 5.8 12.9 12.9v12.9H45.2zm0 6.5c7.1 0 12.9 5.8 12.9 12.9s-5.8 12.9-12.9 12.9H12.9C5.8 58.1 0 52.3 0 45.2s5.8-12.9 12.9-12.9h32.3z" fill="#36c5f0"></path><path d="M97 45.2c0-7.1 5.8-12.9 12.9-12.9s12.9 5.8 12.9 12.9-5.8 12.9-12.9 12.9H97V45.2zm-6.5 0c0 7.1-5.8 12.9-12.9 12.9s-12.9-5.8-12.9-12.9V12.9C64.7 5.8 70.5 0 77.6 0s12.9 5.8 12.9 12.9v32.3z" fill="#2eb67d"></path><path d="M77.6 97c7.1 0 12.9 5.8 12.9 12.9s-5.8 12.9-12.9 12.9-12.9-5.8-12.9-12.9V97h12.9zm0-6.5c-7.1 0-12.9-5.8-12.9-12.9s5.8-12.9 12.9-12.9h32.3c7.1 0 12.9 5.8 12.9 12.9s-5.8 12.9-12.9 12.9H77.6z" fill="#ecb22e"></path></svg>Add to Slack</a>


# Post-install 
Really only relevant for versions of Luxafor-ui < v0.1.0-alpha.2 and distros not supporting .deb or .rpm packages
See [POST-INSTALL.md](./POST-INSTALL.md)

## Build it yourself
Get your Tauri [prerequisites](https://tauri.app/start/prerequisites/) in order first

## Dependencies (Debian/Ubuntu only)
```bash
sudo apt install libgtk-3-dev libgdk3.0-cil-dev libatk1.0-dev libxdo-dev\
 librust-gio-sys-dev librust-pango-sys-dev librust-soup3-sys-dev\
 librust-gdk-pixbuf-sys-dev libjavascriptcoregtk-4.1-dev\
 libwebkit2gtk-4.1-dev \
 libappindicator3-dev # libayatana-appindicator3-dev
```

## NixOS
Use included `shell.nix` (will need tweaking)

Good luck.


## Common steps
Install the Tauri command line interface `tauri-cli`, the wasm-bundler `trunk` and the wasm32 target

Add the wasm32 build target
```bash
rustup target add wasm32-unknown-unknown
```

If on Apple Silicon (M1 or up), install `tauri-cli` and `trunk` from cargo directly.
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

