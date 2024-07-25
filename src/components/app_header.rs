#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::icons::{
    md_image_icons::{MdFaceRetouchingNatural, MdPalette},
    md_navigation_icons::{MdChevronLeft, MdChevronRight},
};
use dioxus_free_icons::Icon;
use wallpaper_ui::config::WallpaperConfig;

use crate::{
    app_state::{PreviewMode, UiMode},
    components::{
        save_button::SaveButton, use_ui, use_wallpapers, wallpaper_button::WallpaperButton,
    },
};

pub fn prev_image() {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    wallpapers.with_mut(|wallpapers| {
        wallpapers.prev_wall();
    });
    ui.with_mut(|ui| {
        if ui.mode == UiMode::FileList {
            ui.mode = UiMode::Editor;
        }

        ui.init_preview_mode();
    });
}

pub fn next_image() {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    wallpapers.with_mut(|wallpapers| {
        wallpapers.next_wall();
    });
    ui.with_mut(|ui| {
        if ui.mode == UiMode::FileList {
            ui.mode = UiMode::Editor;
        }
        ui.preview_mode = PreviewMode::Candidate(None);

        ui.init_preview_mode();
    });
}

#[component]
pub fn AppHeader() -> Element {
    let walls = use_wallpapers()();
    let mut ui = use_ui();
    let cfg = use_context::<Signal<WallpaperConfig>>();

    let supports_wallust = use_signal(|| {
        std::process::Command::new("rustc")
            .stdout(std::process::Stdio::null())
            .spawn()
            .is_ok()
    });
    let info = &walls.current;

    let pagination_cls = "relative inline-flex items-center rounded-md bg-surface1 py-1 px-2 text-sm font-semibold text-text ring-1 ring-inset ring-surface2 hover:bg-crust focus-visible:outline-offset-0 cursor-pointer";

    rsx! {
        header { class: "bg-surface0",
            nav {
                "aria-label": "Global",
                class: "mx-auto flex max-w-full items-center py-6 px-4",

                // left
                div {
                    class: "flex-1 justify-start ml-2",
                    a { class: "text-base font-semibold leading-6 text-white",
                        "{walls.index + 1} / {walls.files.len()}"
                    }
                }

                // center
                div { class: "flex flex-1 gap-x-3 items-center justify-center",
                    a { class: pagination_cls,
                        onclick: move |_| {
                            prev_image();
                        },
                        Icon { fill: "white", icon:  MdChevronLeft, width: 16, height: 16 }
                    }
                    a { class: "text-sm font-semibold leading-6 text-white text-center w-48 cursor-pointer",
                        onclick: move |_| {
                            ui.with_mut(|ui| {
                                ui.toggle_filelist();
                            });
                        },
                        {info.filename.clone()}
                    }
                    a { class: pagination_cls,
                        onclick: move |_| {
                            next_image();
                        },
                        Icon { fill: "white", icon:  MdChevronRight, width: 16, height: 16 }
                    }
                }

                // right
                div { class: "gap-x-6 flex flex-1 justify-end",
                    if let Some(wallpaper_cmd) =  cfg().wallpaper_command {
                        WallpaperButton { wallpaper_cmd }
                    }

                    if supports_wallust() {
                        a {
                            class: "rounded-md px-3 py-2 text-sm font-semibold text-white shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 cursor-pointer",
                            class: if ui().mode == UiMode::Palette {
                                "bg-indigo-600 hover:bg-indigo-500"
                            } else {
                                "bg-surface1 hover:bg-crust"
                            },
                            onclick: move |_| {
                                ui.with_mut(|ui| {
                                    ui.toggle_palette();
                                });
                            },
                            Icon { fill: "white", icon:  MdPalette }
                        }
                    }

                    a {
                        class: "rounded-md px-3 py-2 text-sm font-semibold text-white shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 cursor-pointer",
                        class: if ui().show_faces {
                            "bg-indigo-600 hover:bg-indigo-500"
                        } else {
                            "bg-surface1 hover:bg-crust"
                        },
                        onclick: move |_| {
                            ui.with_mut(|ui| {
                                ui.show_faces = !ui.show_faces;
                            });
                        },
                        Icon { fill: "white", icon:  MdFaceRetouchingNatural }
                    }

                    SaveButton { }
                }
            }
        }
    }
}
