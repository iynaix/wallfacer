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
    text: String,
    geom: Geometry,
    ui: Signal<UiState>,
    wallpapers: Signal<Wallpapers>,
) -> Element {
    let current_geom = (wallpapers)().get_geometry();

    rsx! {
        Button {
            class: class,
            active: current_geom == geom,
            text: text,
            onclick: move |_| {
                wallpapers.with_mut(|wallpapers| {
                    wallpapers.set_geometry(&geom);
                });
                ui.with_mut(|ui| {
                    ui.preview_mode = PreviewMode::None;
                });
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AlignSelectorProps {
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
}

pub fn AlignSelector(mut props: AlignSelectorProps) -> Element {
    let info = (props.wallpapers)().current;
    let ratio = (props.wallpapers)().ratio;
    let align = (props.ui)().preview_mode;
    let geom: Geometry = (props.wallpapers)().get_geometry();
    let (img_w, img_h) = info.image_dimensions();
    let dir = info.direction(&geom);

    rsx! {
        div { class: "flex gap-x-8",
            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                AlignButton {
                    class: "text-sm rounded-l-md",
                    text: "Source",
                    geom: (props.wallpapers)().source.get_geometry(&ratio),
                    wallpapers: props.wallpapers,
                    ui: props.ui,
                }
                AlignButton {
                    class: "text-sm rounded-r-md",
                    text: "Default",
                    geom: info.cropper().crop(&ratio),
                    wallpapers: props.wallpapers,
                    ui: props.ui,
                }
            }

            span {
                class: "isolate inline-flex rounded-md shadow-sm {props.class.unwrap_or_default()}",
                AlignButton {
                    class: "text-sm rounded-l-md",
                    text: if dir == Direction::X { "Left" } else { "Top" },
                    geom: geom.align_start(img_w, img_h),
                    wallpapers: props.wallpapers,
                    ui: props.ui,
                }
                AlignButton {
                    class: "text-sm -ml-px",
                    text: if dir == Direction::X { "Center" } else { "Middle" },
                    geom: geom.align_center(img_w, img_h),
                    wallpapers: props.wallpapers,
                    ui: props.ui,
                }
                AlignButton {
                    class: "text-sm rounded-r-md",
                    text: if dir == Direction::X { "Right" } else { "Bottom" },
                    geom: geom.align_end(img_w, img_h),
                    wallpapers: props.wallpapers,
                    ui: props.ui,
                }
            }

            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                Button {
                    class: "text-sm rounded-md",
                    active: align == PreviewMode::Manual,
                    text: "Manual",
                    onclick: move |_| {
                        props.ui.with_mut(|ui| {
                            ui.preview_mode = if matches!(&ui.preview_mode, PreviewMode::Manual) {
                                PreviewMode::None
                            } else {
                                PreviewMode::Manual
                            }
                        });
                    }
                }
            }
        }
    }
}
