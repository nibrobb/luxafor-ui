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
fn ColorButton(color: &'static str, selected_color: RwSignal<Option<String>>) -> impl IntoView {
    let change_color_action = Action::new(move |input: &String| {
        selected_color.set(Some(input.clone()));
        invoke_set_color(input.clone())
    });

    view! {
        <button
        data-color={color}
        class=move || {
            if selected_color.get().as_ref().is_some_and(|c| c == color) {
                format!("selected {}", color.to_lowercase())
            } else {
                "".to_string()
            }
        }
        on:click=move |_| {
            change_color_action.dispatch(color.to_owned());
        } >
            {color}
        </button>
    }
}


#[component]
pub fn App() -> impl IntoView {
    let selected_color = RwSignal::new(None::<String>);
    view! {
        <main class="container">
            <ColorButton color="Red" selected_color=selected_color/>
            <ColorButton color="Green" selected_color=selected_color/>
            <ColorButton color="Blue" selected_color=selected_color/>
            <ColorButton color="Yellow" selected_color=selected_color/>
            <ColorButton color="Cyan" selected_color=selected_color/>
            <ColorButton color="Magenta" selected_color=selected_color/>
            <ColorButton color="White" selected_color=selected_color/>
            <ColorButton color="Off" selected_color=selected_color/>
        </main>
    }
}
