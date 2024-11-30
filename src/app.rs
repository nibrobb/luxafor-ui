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
pub fn App() -> impl IntoView {
    let change_color = move |color: &str| {
        let color = color.to_string();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&ColorArgs { color: &color }).unwrap();
            invoke("set_light_color", args).await;
        });
    };

    view! {
        <div class="header">
            <div class="row">
                <a href="https://tauri.app" target="_blank">
                    <img width="60" src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
                </a>
                <a href="https://docs.rs/leptos/" target="_blank">
                    <img width="75" src="public/leptos.svg" class="logo leptos" alt="Leptos logo"/>
                </a>
            </div>
        </div>
        <main class="container">
            <button on:click=move |_| change_color("red") >"Red"</button>
            <button on:click=move |_| change_color("green") >"Green"</button>
            <button on:click=move |_| change_color("yellow") >"Yellow"</button>
            <button on:click=move |_| change_color("blue") >"Blue"</button>
            <button on:click=move |_| change_color("cyan") >"Cyan"</button>
            <button on:click=move |_| change_color("magenta") >"Magenta"</button>
            <button on:click=move |_| change_color("white") >"White"</button>
            <button on:click=move |_| change_color("off") >"Turn off"</button>
        </main>
    }
}
