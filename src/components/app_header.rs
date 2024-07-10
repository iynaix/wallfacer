#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::icons::{
    md_device_icons::MdWallpaper,
    md_image_icons::{MdFaceRetouchingNatural, MdPalette},
    md_navigation_icons::{MdChevronLeft, MdChevronRight},
};
use dioxus_free_icons::Icon;
use wallpaper_ui::config::WallpaperConfig;

use crate::{
    app_state::{PreviewMode, UiMode},
    components::{use_ui, use_wallpapers},
};

pub fn save_image() {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    let info = wallpapers().current;
    wallpapers.with_mut(|wallpapers| {
        wallpapers.insert_csv(&info);
        wallpapers.save_csv();

        wallpapers.remove();
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
        ui.is_saving = true;
    });
}

#[component]
pub fn SaveButton() -> Element {
    let mut ui = use_ui();
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
                save_image();
            },
            {btn_text}
        }
    }
}

pub fn prev_image() {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    wallpapers.with_mut(|wallpapers| {
        wallpapers.prev_wall();
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
    });
}

pub fn next_image() {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    wallpapers.with_mut(|wallpapers| {
        wallpapers.next_wall();
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
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
    let wall_path = walls.full_path();

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
                        a {
                            class: "rounded-md px-3 py-2 text-sm font-semibold text-white shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 cursor-pointer",
                            class: "bg-surface1 hover:bg-crust",
                            onclick: move |_| {
                                let wall_cmd = if wallpaper_cmd.contains("$1") {
                                    wallpaper_cmd.replace("$1", &wall_path)
                                } else {
                                    format!("{} {}", wallpaper_cmd, &wall_path)
                                };

                                std::process::Command::new("sh")
                                    .arg("-c")
                                    .arg(wall_cmd)
                                    .spawn()
                                    .expect("failed to set wallpaper");
                            },
                            Icon { fill: "white", icon: MdWallpaper }
                        }
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
