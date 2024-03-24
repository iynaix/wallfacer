#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::{AspectRatio, Direction},
    geometry::Geometry,
    wallpapers::WallInfo,
};

use crate::buttons::Button;

#[derive(Clone, PartialEq, Props)]
pub struct AlignGroupProps {
    class: Option<String>,
    wall_info: Signal<WallInfo>,
    manual_mode: Signal<bool>,
    current_ratio: AspectRatio,
}

pub fn AlignGroup(mut props: AlignGroupProps) -> Element {
    let geom: Geometry = (props.wall_info)().get_geometry(&props.current_ratio);
    let (img_w, img_h) = (props.wall_info)().image_dimensions();
    let dir = (props.wall_info)().direction(&geom);

    rsx! {
        span {
            class: "isolate inline-flex rounded-md shadow-sm {props.class.unwrap_or_default()}",
            Button {
                class: "text-sm rounded-l-md",
                text: "Default",
                onclick: {
                    let ratio = props.current_ratio.clone();
                    move |_| {
                        props.wall_info.with_mut(|info| {
                            info.set_geometry(&ratio, &info.cropper().crop(&ratio));
                        });
                    }
                },
            }
            Button {
                class: "text-sm -ml-px",
                text: if dir == Direction::X { "Left" } else { "Top" },
                onclick: {
                    let geom = geom.clone();
                    let ratio = props.current_ratio.clone();
                    move |_| {
                        props.wall_info.with_mut(|info| {
                            info.set_geometry(&ratio, &geom.align_start(img_w, img_h));
                        });
                    }
                }
            }
            Button {
                class: "text-sm -ml-px",
                text: if dir == Direction::X { "Center" } else { "Middle" },
                onclick: {
                    let geom = geom.clone();
                    let ratio = props.current_ratio.clone();
                    move |_| {
                        props.wall_info.with_mut(|info| {
                            info.set_geometry(&ratio, &geom.align_center(img_w, img_h));
                        });
                    }
                },
            }
            Button {
                class: "text-sm -ml-px",
                text: if dir == Direction::X { "Right" } else { "Bottom" },
                onclick: {
                    let ratio = props.current_ratio.clone();
                    move |_| {
                        props.wall_info.with_mut(|info| {
                            info.set_geometry(&ratio, &geom.align_end(img_w, img_h));
                        });
                    }
                }
            }
            Button {
                class: "text-sm rounded-r-md",
                active: (props.manual_mode)(),
                text: "Manual",
                onclick: move |_| {
                    props.manual_mode.set(!(props.manual_mode)());
                }
            }
        }
    }
}
