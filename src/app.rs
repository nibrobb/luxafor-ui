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

#[derive(Deserialize, Serialize)]
struct ColorArgs<'a> {
    color: &'a str,
}

async fn invoke_set_color(color: String) {
    let color1 = color.to_owned();
    let args = serde_wasm_bindgen::to_value(&ColorArgs { color: &color1 }).unwrap();
    spawn_local(async move {
        invoke("set_light_color", args.clone()).await;
    });
}


#[component]
fn ColorButton(color: &'static str) -> impl IntoView {
    let change_color_action = Action::new(|input: &String| {
        invoke_set_color(input.clone())
    });

    view! {
        <button data-color={color} on:click=move |_| {
            change_color_action.dispatch(color.to_owned());
        } >
            {color}
        </button>
    }
}


#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container">
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
