use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::*;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct ColorArgs<'a> {
    color: &'a str,
}

#[component]
fn ColorButton(color: &'static str) -> impl IntoView {
    let change_color = move |color1: &str| {
        let color2 = color1.to_string();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&ColorArgs { color: &color2 }).unwrap();
            invoke("set_light_color", args).await;
        });
    };
    view! {
        <button data-color={color} on:click=move |_| change_color(color)>
            {color}
        </button>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container">
            <div id="logo-row">
                <a href="https://tauri.app" target="_blank">
                    <img width="60" src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
                </a>
                <a href="https://docs.rs/leptos/" target="_blank">
                    <img width="75" src="public/leptos.svg" class="logo leptos" alt="Leptos logo"/>
                </a>
            </div>
            <ColorButton color="Red"/>
            <ColorButton color="Green"/>
            <ColorButton color="Blue"/>
            <ColorButton color="Yellow"/>
            <ColorButton color="Cyan"/>
            <ColorButton color="Magenta"/>
            <ColorButton color="White"/>
            <ColorButton color="Off"/>
        </main>
    }
}
