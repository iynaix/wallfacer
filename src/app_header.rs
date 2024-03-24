#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::wallpapers::{WallInfo, WallpapersCsv};

use crate::{
    app_state::{UiState, Wallpapers},
    switch::Switch,
};

#[derive(Clone, PartialEq, Eq, Props)]
pub struct SaveButtonProps {
    info: WallInfo,
}

pub fn SaveButton(props: SaveButtonProps) -> Element {
    let mut clicked = use_signal(|| false);

    use_future(move || async move {
        loop {
            if clicked() {
                clicked.set(false);
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let btn_color = if clicked() {
        "bg-green-600"
    } else {
        "bg-indigo-600"
    };
    let btn_text = if clicked() { "Saved" } else { "Save" };

    rsx! {
        a {
            class: "rounded-md px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer {btn_color}",
            onclick: {
                let info = props.info;
                move |_| {
                    let mut wallpapers_csv = WallpapersCsv::new();
                    wallpapers_csv.insert(info.filename.clone(), info.clone());
                    wallpapers_csv.save();

                    clicked.set(true);
                }
            },
            {btn_text}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AppHeaderProps {
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
}

#[allow(clippy::too_many_lines)]
pub fn AppHeader(mut props: AppHeaderProps) -> Element {
    let info = (props.wallpapers)().current;

    let pagination_cls = "relative inline-flex items-center rounded-md bg-surface1 py-1 px-2 text-sm font-semibold text-text ring-1 ring-inset ring-surface2 hover:bg-oveylay0 focus-visible:outline-offset-0 cursor-pointer";

    rsx! {
        header { class: "bg-surface0",
            nav {
                "aria-label": "Global",
                class: "mx-auto flex max-w-full items-center justify-between py-6 px-4",
                div { class: "flex gap-x-4 items-center",
                    a { class: pagination_cls,
                        onclick: move |_| {
                            props.wallpapers.with_mut(|wallpapers| {
                                wallpapers.prev_wall();
                            });
                        },
                        "<"
                    }
                    a { class: "text-sm font-semibold leading-6 text-white",
                        onclick: move |_| {
                            props.ui.with_mut(|ui| {
                                ui.show_filelist = !ui.show_filelist;
                            });
                        },
                        {info.filename.clone()}
                    }
                    a { class: pagination_cls,
                        onclick: move |_| {
                            props.wallpapers.with_mut(|wallpapers| {
                                wallpapers.next_wall();
                            });
                        },
                        ">"
                    }
                    // done checkbox
                    a {
                        class: "rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer",
                        onclick: move |_| {
                            props.wallpapers.with_mut(|wallpapers| {
                                wallpapers.remove();
                            });
                        },
                        "Done"
                    }
                }
                div { class: "gap-8 flex flex-1 justify-end",
                    Switch {
                        label: rsx! {
                            span {
                                class: "ml-3 text-md",
                                span {
                                    class: "font-medium text-white",
                                    "Faces ({info.faces.len()})"
                                }
                            }
                        },
                        value: (props.ui)().show_faces,
                        onchange: move |_| {
                            props.ui.with_mut(|ui| {
                                ui.show_faces = !ui.show_faces;
                            });
                        }
                    },

                    SaveButton {
                        info: info,
                    }
                }
            }
        }
    }
}
