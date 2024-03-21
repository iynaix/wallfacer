use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    class: Option<String>,
    text: String,
    onclick: EventHandler<MouseEvent>,
}

pub fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            "type": "button",
            class: "relative inline-flex items-center bg-mantle px-3 py-2 font-semibold text-text ring-1 ring-inset ring-gray-300 hover:bg-crust focus:z-10 {props.class.unwrap_or_default()}",
            onclick: move |evt| props.onclick.call(evt),
            {props.text},
        }
    }
}
