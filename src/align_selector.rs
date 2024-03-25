#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{cropper::Direction, geometry::Geometry};

use crate::{
    app_state::{AlignMode, UiState, Wallpapers},
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
    let ratio = (props.ui)().ratio;
    let align = (props.ui)().align_mode;
    let geom: Geometry = info.get_geometry(&(props.ui)().ratio);
    let (img_w, img_h) = info.image_dimensions();
    let dir = info.direction(&geom);

    let set_alignment = |geom: Geometry, align: AlignMode| {
        move |_| {
            props.ui.with_mut(|ui| {
                ui.preview_geometry = None;
                ui.align_mode = align.clone();

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
                active: align == AlignMode::Source,
                text: "Source",
                onclick: set_alignment((props.wallpapers)().source.get_geometry(&ratio), AlignMode::Source),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == AlignMode::Default,
                text: "Default",
                onclick: set_alignment(info.cropper().crop(&ratio), AlignMode::Default),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == AlignMode::Start,
                text: if dir == Direction::X { "Left" } else { "Top" },
                onclick: set_alignment(geom.align_start(img_w, img_h), AlignMode::Start),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == AlignMode::Center,
                text: if dir == Direction::X { "Center" } else { "Middle" },
                onclick: set_alignment(geom.align_center(img_w, img_h), AlignMode::Center),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == AlignMode::End,
                text: if dir == Direction::X { "Right" } else { "Bottom" },
                onclick: set_alignment(geom.align_end(img_w, img_h), AlignMode::End),
            }
            Button {
                class: "text-sm rounded-r-md",
                active: align == AlignMode::Manual,
                text: "Manual",
                onclick: move |_| {
                    props.ui.with_mut(|ui| {
                        if ui.align_mode == AlignMode::Manual {
                            ui.align_mode = AlignMode::None;
                        } else {
                            ui.align_mode = AlignMode::Manual;
                            ui.preview_geometry = Some(geom.clone());
                        }
                    });
                }
            }
        }
    }
}
