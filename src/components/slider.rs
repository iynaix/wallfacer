#![allow(non_snake_case)]
use dioxus::prelude::*;

#[component]
pub fn Slider(
    name: String,
    class: Option<String>,
    value: u8,
    onchange: EventHandler<u8>,
) -> Element {
    let label_id = format!("{name}-label");
    let label = format!("{} ({})", name, value);

    rsx! {
        div {
            class: class.unwrap_or_default(),
            label {
                r#for: label_id.clone(),
                class: "block text-ctp-base font-bold leading-6 text-ctp-text",
                {label}
            }
            input {
                id: label_id,
                r#type: "range",
                value: value.to_string(),
                class: "w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700",
                onchange: move |evt| {
                    let raw_value = evt.value();
                    let value = raw_value.parse::<u8>().unwrap_or_else(|_| panic!("invalid value for {name}: {raw_value}"));
                    onchange.call(value);
                }
            }
        }
    }
}
