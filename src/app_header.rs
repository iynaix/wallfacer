#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::icons::{
    md_action_icons::MdDone,
    md_image_icons::MdFaceRetouchingNatural,
    md_navigation_icons::{MdChevronLeft, MdChevronRight},
};
use dioxus_free_icons::Icon;
use wallpaper_ui::wallpapers::WallpapersCsv;

use crate::app_state::{UiState, Wallpapers};

#[component]
pub fn SaveButton(wallpapers: Signal<Wallpapers>) -> Element {
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
            class: "rounded-md px-5 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer",
            class: btn_color,
            onclick: {
                move |_| {
                    let info = wallpapers().current;
                    let mut wallpapers_csv = WallpapersCsv::new();
                    wallpapers_csv.insert(info.filename.clone(), info);
                    wallpapers_csv.save();

                    // source has now been updated!
                    wallpapers.with_mut(|wallpapers| {
                        wallpapers.source = wallpapers.current.clone();
                    });

                    clicked.set(true);
                }
            },
            {btn_text}
        }
    }
}

#[component]
pub fn AppHeader(wallpapers: Signal<Wallpapers>, ui: Signal<UiState>) -> Element {
    let info = wallpapers().current;

    let pagination_cls = "relative inline-flex items-center rounded-md bg-surface1 py-1 px-2 text-sm font-semibold text-text ring-1 ring-inset ring-surface2 hover:bg-crust focus-visible:outline-offset-0 cursor-pointer";

    rsx! {
        header { class: "bg-surface0",
            nav {
                "aria-label": "Global",
                class: "mx-auto flex max-w-full items-center justify-between py-6 px-4",
                div { class: "flex gap-x-3 items-center",
                    a { class: pagination_cls,
                        onclick: move |_| {
                            wallpapers.with_mut(|wallpapers| {
                                wallpapers.prev_wall();
                            });
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
                            wallpapers.with_mut(|wallpapers| {
                                wallpapers.next_wall();
                            });
                        },
                        Icon { fill: "white", icon:  MdChevronRight, width: 16, height: 16 }
                    }
                    // done checkbox
                    a {
                        class: "rounded-md bg-indigo-600 ml-3 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer",
                        onclick: move |_| {
                            wallpapers.with_mut(|wallpapers| {
                                wallpapers.remove();
                            });
                        },
                        Icon { fill: "white", icon:  MdDone }
                    }
                }
                div { class: "gap-8 flex flex-1 justify-end",
                    a {
                        class: "rounded-md ml-8 px-3 py-2 text-sm font-semibold text-white shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 cursor-pointer",
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
                    SaveButton { wallpapers }
                }
            }
        }
    }
}
