#![allow(non_snake_case)]
use dioxus::prelude::*;

#[component]
pub fn Button(
    class: Option<String>,
    active: Option<bool>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmouseenter: Option<EventHandler<MouseEvent>>,
    onmouseleave: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let active_cls = if active.unwrap_or_default() {
        "bg-active"
    } else {
        "bg-surface0"
    };

    rsx! {
        button {
            r#type: "button",
            class: "relative inline-flex items-center px-3 py-2 font-semibold text-text ring-1 ring-inset ring-surface1 hover:bg-crust focus:z-10 {active_cls} {class.unwrap_or_default()}",
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            onmouseenter: move |evt| {
                if let Some(handler) = &onmouseenter {
                    handler.call(evt);
                }
            },
            onmouseleave: move |evt| {
                if let Some(handler) = &onmouseleave {
                    handler.call(evt);
                }
            },
            {children},
        }
    }
}
