#![allow(non_snake_case)]
use dioxus::prelude::*;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Props)]
pub struct SwitchProps {
    label: String,
    checked: Signal<bool>,
}

pub fn Switch(mut props: SwitchProps) -> Element {
    let check_bg = if (props.checked)() {
        "bg-indigo-600"
    } else {
        "bg-gray-200"
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
                "type": "button",
                class: "{check_bg} relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2",
                role: "switch",
                aria_checked: "{props.checked}",
                aria_labelledby: "show-faces",
                onclick: move |_| {
                    props.checked.with_mut(|checked| *checked = !*checked);
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

/*
<div class="flex items-center">
  <!-- Enabled: "bg-indigo-600", Not Enabled: "bg-gray-200" -->
  <button type="button" class="bg-gray-200 relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2" role="switch" aria-checked="false" aria-labelledby="annual-billing-label">
    <!-- Enabled: "translate-x-5", Not Enabled: "translate-x-0" -->
    <span aria-hidden="true" class="translate-x-0 pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"></span>
  </button>
  <span class="ml-3 text-sm" id="annual-billing-label">
    <span class="font-medium text-gray-900">Annual billing</span>
    <span class="text-gray-500">(Save 10%)</span>
  </span>
</div>
*/
