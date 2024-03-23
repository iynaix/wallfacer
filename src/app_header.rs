#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::wallpapers::{WallInfo, WallpapersCsv};

use crate::switch::Switch;

#[derive(Clone, PartialEq, Props)]
pub struct AppHeaderProps {
    show_faces: Signal<bool>,
    show_filelist: Signal<bool>,
    wall_info: WallInfo,
}

pub fn AppHeader(mut props: AppHeaderProps) -> Element {
    rsx! {
        header { class: "bg-gray-900",
            nav {
                "aria-label": "Global",
                class: "mx-auto flex max-w-full items-center justify-between py-6 px-4",
                div { class: "flex lg:hidden",
                    button {
                        r#type: "button",
                        class: "-m-2.5 inline-flex items-center justify-center rounded-md p-2.5 text-gray-400",
                        span { class: "sr-only", "Open main menu" }
                        svg {
                            "aria-hidden": "true",
                            "stroke": "currentColor",
                            "stroke-width": "1.5",
                            "fill": "none",
                            "viewBox": "0 0 24 24",
                            class: "h-6 w-6",
                            path {
                                "stroke-linecap": "round",
                                "stroke-linejoin": "round",
                                "d": "M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
                            }
                        }
                    }
                }
                div { class: "hidden lg:flex lg:gap-x-12",
                    a {
                        class: "text-sm font-semibold leading-6 text-white",
                        onclick: move |_| {
                            props.show_filelist.set(!(props.show_filelist)());
                        },
                        {props.wall_info.filename.clone()}
                    }
                }
                div { class: "hidden gap-8 lg:flex lg:flex-1 lg:justify-end",
                    Switch {
                        label: "Faces ({props.wall_info.faces.len()})",
                        checked: props.show_faces,
                    },

                    a {
                        class: "rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600",
                        onclick: {
                            let info = props.wall_info;
                            move |_| {
                                let mut wallpapers = WallpapersCsv::new();
                                wallpapers.insert(info.filename.clone(), info.clone());
                                wallpapers.save();
                            }
                        },
                        "Save"
                    }
                }
            }
        }
    }
}
