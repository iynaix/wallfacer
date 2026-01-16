#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DropdownOptions<T> {
    pub values: Vec<T>,
}

impl<T: Copy + PartialEq + ToString + std::fmt::Display> DropdownOptions<T> {
    pub const fn new(values: Vec<T>) -> Self {
        Self { values }
    }

    pub fn from_string(&self, value: &str) -> T {
        *self
            .values
            .iter()
            .find(|x| x.to_string() == value)
            .unwrap_or_else(|| panic!("invalid value: {}", value))
    }

    pub fn position(&self, value: T) -> usize {
        self.values
            .iter()
            .position(|x| x == &value)
            .unwrap_or_else(|| panic!("invalid position for {value}"))
    }
}

#[component]
pub fn Dropdown<T: Copy + PartialEq + ToString + std::fmt::Display + 'static>(
    name: String,
    class: Option<String>,
    options: DropdownOptions<T>,
    value: T,
    onchange: EventHandler<T>,
) -> Element {
    let label_id = format!("{name}-label");
    let mut open = use_signal(|| false);
    let selected_index = options.position(value) + 1;

    rsx! {
        div {
            class: class.unwrap_or_default(),
            label {
                class: "block font-bold leading-6 text-ctp-text",
                id: label_id.clone(),
                {name}
            }
            div { class: "relative mt-2",
                button {
                    r#type: "button",
                    "aria-labelledby": label_id,
                    "aria-expanded": "true",
                    "aria-haspopup": "listbox",
                    class: "relative w-full cursor-default rounded-md bg-white py-1.5 pl-3 pr-10 text-left text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 focus:outline-none focus:ring-2 focus:ring-indigo-600 sm:text-sm sm:leading-6",
                    onclick: move |_| {
                        open.set(!open());
                    },
                    span { class: "block truncate", {value.to_string()} }
                    span { class: "pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2",
                        svg {
                            "aria-hidden": "true",
                            "viewBox": "0 0 20 20",
                            "fill": "currentColor",
                            class: "h-5 w-5 text-gray-400",
                            path {
                                "d": "M10 3a.75.75 0 01.55.24l3.25 3.5a.75.75 0 11-1.1 1.02L10 4.852 7.3 7.76a.75.75 0 01-1.1-1.02l3.25-3.5A.75.75 0 0110 3zm-3.76 9.2a.75.75 0 011.06.04l2.7 2.908 2.7-2.908a.75.75 0 111.1 1.02l-3.25 3.5a.75.75 0 01-1.1 0l-3.25-3.5a.75.75 0 01.04-1.06z",
                                "fill-rule": "evenodd",
                                "clip-rule": "evenodd"
                            }
                        }
                    }
                }
                if open() {
                    ul {
                        role: "listbox",
                        "aria-activedescendant": "listbox-option-{selected_index}",
                        tabindex: "-1",
                        "aria-labelledby": label_id.clone(),
                        class: "absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-ctp-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm",
                        for opt in options.values {
                            li {
                                role: "option",
                                class: "text-gray-900 relative cursor-default select-none py-2 pl-3 pr-9",
                                onclick: move |_| {
                                    onchange.call(opt);
                                    open.set(false);
                                },
                                span { class: "font-normal block truncate", {opt.to_string()} }
                                span { class: "text-indigo-600 absolute inset-y-0 right-0 flex items-center pr-4",
                                    class: if opt == value { "" } else { "hidden" },
                                    svg {
                                        "aria-hidden": "true",
                                        "viewBox": "0 0 20 20",
                                        "fill": "currentColor",
                                        class: "h-5 w-5",
                                        path {
                                            "d": "M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z",
                                            "clip-rule": "evenodd",
                                            "fill-rule": "evenodd"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
