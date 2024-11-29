use leptos::leptos_dom::ev::SubmitEvent;
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
    let (color, set_color) = create_signal(String::new());

    let update_color = move |ev| {
        let v = event_target_value(&ev);
        set_color.set(v);
    };
    let on_set_color = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let color = color.get_untracked();
            if color.is_empty() {
                return;
            }

            let args = serde_wasm_bindgen::to_value(&ColorArgs { color: &color }).unwrap();
            // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
            let color = invoke("set_light_color", args).await.as_string().unwrap();
            set_color.set(color);
        });
    };
    view! {
        <div class="header">
            <div class="row">
                <a href="https://tauri.app" target="_blank">
                    <img width="75" src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
                </a>
                <a href="https://docs.rs/leptos/" target="_blank">
                    <img width="75" src="public/leptos.svg" class="logo leptos" alt="Leptos logo"/>
                </a>
            </div>
        </div>
        <main class="container">
            <form class="row" on:submit=on_set_color>
                <input
                    id="luxafor-color"
                    placeholder="Enter a name..."
                    on:input=update_color
                />
                <button type="submit">"Greet"</button>
            </form>
            <p>{ move || color.get() }</p>
        </main>
    }
}
