#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct SwitchProps {
    label: String,
    checked: Signal<bool>,
}

pub fn Switch(mut props: SwitchProps) -> Element {
    let check_bg = if (props.checked)() {
        "bg-active"
    } else {
        "bg-surface0"
    };

    let check_translate = if (props.checked)() {
        "translate-x-5"
    } else {
        "translate-x-0"
    };

    rsx! {
        div {
            class: "flex items-center",
            button {
                r#type: "button",
                class: "{check_bg} relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2",
                role: "switch",
                aria_checked: "{props.checked}",
                aria_labelledby: "show-faces",
                onclick: move |_| {
                    props.checked.set(!(props.checked)());
                },
                span {
                    class: "{check_translate} pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out",
                    aria_hidden: "true",
                }
            }
            span {
                class: "ml-3 text-sm",
                span {
                    class: "font-medium text-text",
                    {props.label}
                }
            }
        }
    }
}
