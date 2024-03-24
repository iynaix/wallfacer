#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{cropper::Direction, geometry::Geometry, wallpapers::WallInfo};

use crate::{app_state::UiState, buttons::Button};

#[derive(Clone, PartialEq, Props)]
pub struct AlignSelectorProps {
    class: Option<String>,
    wall_info: Signal<WallInfo>,
    ui: Signal<UiState>,
}

pub fn AlignSelector(mut props: AlignSelectorProps) -> Element {
    let geom: Geometry = (props.wall_info)().get_geometry(&(props.ui)().ratio);
    let (img_w, img_h) = (props.wall_info)().image_dimensions();
    let dir = (props.wall_info)().direction(&geom);

    let set_alignment = |geom: Geometry| {
        move |_| {
            props.ui.with_mut(|ui| {
                ui.preview_geometry = None;
                ui.manual_mode = false;

                props.wall_info.with_mut(|info| {
                    info.set_geometry(&ui.ratio, &geom);
                });
            });
        }
    };

    rsx! {
        span {
            class: "isolate inline-flex rounded-md shadow-sm {props.class.unwrap_or_default()}",
            Button {
                class: "text-sm rounded-l-md",
                text: "Default",
                onclick: set_alignment((props.wall_info)().cropper().crop(&(props.ui)().ratio)),
            }
            Button {
                class: "text-sm -ml-px",
                text: if dir == Direction::X { "Left" } else { "Top" },
                onclick: set_alignment(geom.align_start(img_w, img_h)),
            }
            Button {
                class: "text-sm -ml-px",
                text: if dir == Direction::X { "Center" } else { "Middle" },
                onclick: set_alignment(geom.align_center(img_w, img_h)),
            }
            Button {
                class: "text-sm -ml-px",
                text: if dir == Direction::X { "Right" } else { "Bottom" },
                onclick: set_alignment(geom.align_center(img_w, img_h)),
            }
            Button {
                class: "text-sm rounded-r-md",
                active: (props.ui)().manual_mode,
                text: "Manual",
                onclick: move |_| {
                    props.ui.with_mut(|ui| {
                        ui.manual_mode = !ui.manual_mode;
                    });
                }
            }
        }
    }
}
