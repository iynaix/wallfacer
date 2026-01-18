#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_editor_icons::{
    MdFormatAlignCenter, MdFormatAlignLeft, MdFormatAlignRight, MdVerticalAlignBottom,
    MdVerticalAlignCenter, MdVerticalAlignTop,
};

use crate::components::button::PreviewableButton;
use crate::state::Wall;
use wallfacer::{cropper::Direction, geometry::Geometry};

#[component]
fn AlignButton(wall: Signal<Wall>, class: String, geom: Geometry, children: Element) -> Element {
    let current_geom = wall().get_geometry();

    rsx! {
        PreviewableButton {
            wall,
            geom: geom.clone(),
            class,
            active: current_geom == geom,
            onclick: move |_| {
                wall.with_mut(|wall| wall.set_geometry(&geom));
            },
            {children}
        }
    }
}

#[component]
pub fn AlignButtons(wall: Signal<Wall>, class: Option<String>) -> Element {
    let Wall {
        current: info,
        ratio,
        ..
    } = wall();
    let geom = wall().get_geometry();
    let dir = info.direction(&geom);

    rsx! {
            span {
                class: "isolate rounded-md shadow-sm",
                AlignButton {
                    wall,
                    class: "text-sm rounded-l-md",
                    geom: wall().source.get_geometry(&ratio),
                    "Source"
                }
                AlignButton {
                    wall,
                    class: "text-sm rounded-r-md",
                    geom: info.cropper().crop(&ratio),
                    "Default"
                }
            }

            span {
                class: format!("isolate rounded-md shadow-sm {}", class.unwrap_or_default()),
                AlignButton {
                    wall,
                    class: "text-sm rounded-l-md",
                    geom: geom.align_start(info.width, info.height),
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignLeft }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignTop }
                    }
                }
                AlignButton {
                    wall,
                    class: "text-sm -ml-px",
                    geom: geom.align_center(info.width, info.height),
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignCenter }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignCenter }
                    }
               }
                AlignButton {
                    wall,
                    class: "text-sm rounded-r-md",
                    geom: geom.align_end(info.width, info.height),
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignRight }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignBottom }
                    }
                }
            }
    }
}
