#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::icons::{
    md_image_icons::{MdFaceRetouchingNatural, MdPalette},
    md_navigation_icons::{MdChevronLeft, MdChevronRight},
};
use dioxus_free_icons::Icon;
use wallpaper_ui::wallpapers::WallpapersCsv;

use crate::app_state::{PreviewMode, UiState, Wallpapers};

pub fn save_image(wallpapers: &mut Signal<Wallpapers>, ui: &mut Signal<UiState>) {
    let info = wallpapers().current;
    let mut wallpapers_csv = WallpapersCsv::load();
    wallpapers_csv.insert(info.filename.clone(), info);
    let resolutions: Vec<_> = wallpapers()
        .resolutions
        .iter()
        .map(|(_, ratio)| ratio.clone())
        .collect();
    wallpapers_csv.save(&resolutions);

    wallpapers.with_mut(|wallpapers| {
        wallpapers.remove();
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
        ui.is_saving = true;
    });
}

#[component]
pub fn SaveButton(wallpapers: Signal<Wallpapers>, ui: Signal<UiState>) -> Element {
    let clicked = ui().is_saving;

    use_future(move || async move {
        loop {
            if ui().is_saving {
                ui.with_mut(|ui| {
                    ui.is_saving = false;
                });
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let btn_color = if clicked {
        "bg-green-600"
    } else {
        "bg-indigo-600"
    };
    let btn_text = if clicked { "Saved" } else { "Save" };

    rsx! {
        a {
            class: "rounded-md px-5 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer",
            class: btn_color,
            onclick: move |_| {
                save_image(&mut wallpapers, &mut ui);
            },
            {btn_text}
        }
    }
}

pub fn prev_image(wallpapers: &mut Signal<Wallpapers>, ui: &mut Signal<UiState>) {
    wallpapers.with_mut(|wallpapers| {
        wallpapers.prev_wall();
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
    });
}

pub fn next_image(wallpapers: &mut Signal<Wallpapers>, ui: &mut Signal<UiState>) {
    wallpapers.with_mut(|wallpapers| {
        wallpapers.next_wall();
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
    });
}

#[component]
pub fn AppHeader(wallpapers: Signal<Wallpapers>, ui: Signal<UiState>) -> Element {
    let supports_wallust = use_signal(|| {
        std::process::Command::new("rustc")
            .stdout(std::process::Stdio::null())
            .spawn()
            .is_ok()
    });
    let info = wallpapers().current;

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
                        "{wallpapers().index + 1} / {wallpapers().files.len()}"
                    }
                }

                // center
                div { class: "flex flex-1 gap-x-3 items-center justify-center",
                    a { class: pagination_cls,
                        onclick: move |_| {
                            prev_image(&mut wallpapers, &mut ui);
                        },
                        Icon { fill: "white", icon:  MdChevronLeft, width: 16, height: 16 }
                    }
                    a { class: "text-sm font-semibold leading-6 text-white text-center w-48 cursor-pointer",
                        onclick: move |_| {
                            ui.with_mut(|ui| {
                                ui.show_filelist = !ui.show_filelist;
                            });
                        },
                        {info.filename}
                    }
                    a { class: pagination_cls,
                        onclick: move |_| {
                            next_image(&mut wallpapers, &mut ui);
                        },
                        Icon { fill: "white", icon:  MdChevronRight, width: 16, height: 16 }
                    }
                }

                // right
                div { class: "gap-x-6 flex flex-1 justify-end",
                    if supports_wallust() {
                        a {
                            class: "rounded-md px-3 py-2 text-sm font-semibold text-white shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 cursor-pointer",
                            class: if ui().show_palette {
                                "bg-indigo-600 hover:bg-indigo-500"
                            } else {
                                "bg-surface1 hover:bg-crust"
                            },
                            onclick: move |_| {
                                ui.with_mut(|ui| {
                                    ui.show_palette = !ui.show_palette;
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

                    SaveButton { wallpapers, ui }
                }
            }
        }
    }
}
