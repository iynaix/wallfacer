#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DropdownOptions<T> {
    pub values: Vec<T>,
    label_fn: fn(&T) -> String,
}

impl<T: std::fmt::Display + Clone> DropdownOptions<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self {
            values,
            label_fn: std::string::ToString::to_string,
        }
    }

    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_label(self, label_fn: fn(&T) -> String) -> Self {
        Self { label_fn, ..self }
    }

    pub fn from_value(&self, value: &str) -> T {
        self.values
            .iter()
            .find(|x| x.to_string() == value)
            .unwrap_or_else(|| panic!("invalid value: {}", value))
            .clone()
    }
}

#[component]
pub fn Dropdown<T: Clone + PartialEq + std::fmt::Display + 'static>(
    name: String,
    options: DropdownOptions<T>,
    value: String,
    onchange: EventHandler<T>,
) -> Element {
    rsx! {
        div {
            label {
                r#for: name.to_string(),
                class: "block text-sm font-medium leading-6 text-gray-900",
                {name.to_string()}
            }
            select {
                name,
                class: "mt-2 block w-full rounded-md border-0 py-1.5 pl-3 pr-10 text-gray-900 ring-1 ring-inset ring-gray-300 focus:ring-2 focus:ring-indigo-600 sm:text-sm sm:leading-6",
                onchange: move |evt| {
                    let new_value = options.from_value(evt.value().as_str());

                    onchange.call(new_value);
                },
                for (label, opt_value) in options
                    .values
                    .iter()
                    .map(|v| ((options.label_fn)(v), v.to_string()))
                {
                    option {
                        label,
                        value: opt_value.clone(),
                        selected: opt_value == value,
                    }
                }
            }
        }
    }
}
