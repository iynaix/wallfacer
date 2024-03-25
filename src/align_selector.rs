#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{cropper::Direction, geometry::Geometry};

use crate::{
    app_state::{UiState, Wallpapers},
    buttons::Button,
};

#[derive(Clone, PartialEq, Props)]
pub struct AlignSelectorProps {
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
}

pub fn AlignSelector(mut props: AlignSelectorProps) -> Element {
    let info = (props.wallpapers)().current;
    let geom: Geometry = info.get_geometry(&(props.ui)().ratio);
    let (img_w, img_h) = info.image_dimensions();
    let dir = info.direction(&geom);

    let set_alignment = |geom: Geometry| {
        move |_| {
            props.ui.with_mut(|ui| {
                ui.preview_geometry = None;
                ui.manual_mode = false;

                props.wallpapers.with_mut(|wallpapers| {
                    wallpapers.current.set_geometry(&ui.ratio, &geom);
                });
            });
        }
    };

    rsx! {
        span {
            class: "isolate inline-flex rounded-md shadow-sm {props.class.unwrap_or_default()}",
            Button {
                class: "text-sm rounded-l-md",
                text: "Source",
                onclick: set_alignment((props.wallpapers)().source.get_geometry(&(props.ui)().ratio)),
            }
            Button {
                class: "text-sm -ml-px",
                text: "Default",
                onclick: set_alignment(info.cropper().crop(&(props.ui)().ratio)),
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
                onclick: set_alignment(geom.align_end(img_w, img_h)),
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
