use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    class: Option<String>,
    text: String,
    onclick: Option<EventHandler<MouseEvent>>,
    onmouseenter: Option<EventHandler<MouseEvent>>,
    onmouseleave: Option<EventHandler<MouseEvent>>,
}

pub fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            "type": "button",
            class: "relative inline-flex items-center bg-mantle px-3 py-2 font-semibold text-text ring-1 ring-inset ring-gray-300 hover:bg-crust focus:z-10 {props.class.unwrap_or_default()}",
            onclick: move |evt| {
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },
            onmouseenter: move |evt| {
                if let Some(handler) = &props.onmouseenter {
                    handler.call(evt);
                }
            },
            onmouseleave: move |evt| {
                if let Some(handler) = &props.onmouseleave {
                    handler.call(evt);
                }
            },
            {props.text},
        }
    }
}
