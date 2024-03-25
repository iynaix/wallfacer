#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{cropper::Direction, geometry::Geometry};

use crate::{
    app_state::{PreviewMode, UiState, Wallpapers},
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
    let align = (props.ui)().preview_mode;
    let geom: Geometry = info.get_geometry(&(props.ui)().ratio);
    let (img_w, img_h) = info.image_dimensions();
    let dir = info.direction(&geom);

    let set_alignment = |geom: Geometry, align: PreviewMode| {
        move |_| {
            props.ui.with_mut(|ui| {
                ui.preview_mode = align.clone();

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
                active: align == PreviewMode::Source,
                text: "Source",
                onclick: set_alignment((props.wallpapers)().source.get_geometry(&ratio), PreviewMode::Source),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == PreviewMode::Default,
                text: "Default",
                onclick: set_alignment(info.cropper().crop(&ratio), PreviewMode::Default),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == PreviewMode::Start,
                text: if dir == Direction::X { "Left" } else { "Top" },
                onclick: set_alignment(geom.align_start(img_w, img_h), PreviewMode::Start),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == PreviewMode::Center,
                text: if dir == Direction::X { "Center" } else { "Middle" },
                onclick: set_alignment(geom.align_center(img_w, img_h), PreviewMode::Center),
            }
            Button {
                class: "text-sm -ml-px",
                active: align == PreviewMode::End,
                text: if dir == Direction::X { "Right" } else { "Bottom" },
                onclick: set_alignment(geom.align_end(img_w, img_h), PreviewMode::End),
            }
            Button {
                class: "text-sm rounded-r-md",
                active: align.is_manual(),
                text: "Manual",
                onclick: move |_| {
                    props.ui.with_mut(|ui| {
                        ui.preview_mode = if let PreviewMode::Manual(manual_geom) = &ui.preview_mode {
                            props.wallpapers.with_mut(|wallpapers| {
                                wallpapers.current.set_geometry(&ui.ratio, manual_geom);
                            });

                            PreviewMode::None
                        } else {
                            PreviewMode::Manual(geom.clone())
                        }
                    });
                }
            }
        }
    }
}
