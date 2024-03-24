#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct SwitchProps {
    label: Element,
    value: bool,
    onchange: EventHandler<MouseEvent>,
}

pub fn Switch(props: SwitchProps) -> Element {
    let check_bg = if props.value {
        "bg-active"
    } else {
        "bg-surface0"
    };

    let check_translate = if props.value {
        "translate-x-5"
    } else {
        "translate-x-0"
    };

    rsx! {
        div {
            class: "flex items-center",
            button {
                r#type: "button",
                class: "{check_bg} relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-surface2 transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2",
                role: "switch",
                aria_checked: "{props.value}",
                aria_labelledby: "show-faces",
                onclick: move |evt| {
                    props.onchange.call(evt);
                },
                span {
                    class: "{check_translate} pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out",
                    aria_hidden: "true",
                }
            }
            {props.label}
        }
    }
}
