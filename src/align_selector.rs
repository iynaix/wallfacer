#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{cropper::Direction, geometry::Geometry};

use crate::{
    app_state::{PreviewMode, UiState, Wallpapers},
    buttons::Button,
};

#[component]
fn AlignButton(
    class: String,
    geom: Geometry,
    ui: Signal<UiState>,
    wallpapers: Signal<Wallpapers>,
    children: Element,
) -> Element {
    let current_geom = (wallpapers)().get_geometry();

    rsx! {
        Button {
            class,
            active: current_geom == geom,
            onclick: move |_| {
                wallpapers.with_mut(|wallpapers| {
                    wallpapers.set_geometry(&geom);
                });
                ui.with_mut(|ui| {
                    ui.preview_mode = PreviewMode::Candidate(None);
                });
            },
            {children}
        }
    }
}

#[component]
pub fn AlignSelector(
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
) -> Element {
    let info = wallpapers().current;
    let ratio = wallpapers().ratio;
    let align = ui().preview_mode;
    let geom: Geometry = wallpapers().get_geometry();
    let (img_w, img_h) = info.image_dimensions();
    let dir = info.direction(&geom);

    rsx! {
        div { class: "flex gap-x-8",
            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                AlignButton {
                    class: "text-sm rounded-l-md",
                    geom: wallpapers().source.get_geometry(&ratio),
                    wallpapers,
                    ui,
                    "Source"
                }
                AlignButton {
                    class: "text-sm rounded-r-md",
                    geom: info.cropper().crop(&ratio),
                    wallpapers,
                    ui,
                    "Default"
                }
            }

            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                class: class.unwrap_or_default(),
                AlignButton {
                    class: "text-sm rounded-l-md",
                    geom: geom.align_start(img_w, img_h),
                    wallpapers,
                    ui,
                    {if dir == Direction::X { "Left" } else { "Top" }}
                }
                AlignButton {
                    class: "text-sm -ml-px",
                    geom: geom.align_center(img_w, img_h),
                    wallpapers,
                    ui,
                    {if dir == Direction::X { "Center" } else { "Middle" }}
                }
                AlignButton {
                    class: "text-sm rounded-r-md",
                    geom: geom.align_end(img_w, img_h),
                    wallpapers,
                    ui,
                    {if dir == Direction::X { "Right" } else { "Bottom" }}
                }
            }

            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                Button {
                    class: "text-sm rounded-md",
                    active: align == PreviewMode::Manual,
                    onclick: move |_| {
                        ui.with_mut(|ui| {
                            ui.preview_mode = if matches!(&ui.preview_mode, PreviewMode::Manual) {
                                PreviewMode::Candidate(None)
                            } else {
                                PreviewMode::Manual
                            }
                        });
                    }
                    {"Manual"}
                }
            }
        }
    }
}
