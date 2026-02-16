#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_editor_icons::{
    MdFormatAlignCenter, MdFormatAlignLeft, MdFormatAlignRight, MdVerticalAlignBottom,
    MdVerticalAlignCenter, MdVerticalAlignTop,
};
use wallfacer::cropper::{Cropper, Direction};

use crate::components::button::PreviewableButton;
use crate::state::Wall;

#[derive(Debug, Clone, PartialEq, Eq)]
enum AlignType {
    Source,
    Default,
    Start,
    Center,
    End,
}

#[component]
fn AlignButton(
    wall: Signal<Wall>,
    class: String,
    align_type: AlignType,
    children: Element,
) -> Element {
    let current_geom = wall().get_current_geometry();

    let w = wall().current.width;
    let h = wall().current.height;
    let geom = match align_type {
        AlignType::Source => wall().get_current_geometry(),
        AlignType::Default => wall().current.cropper().crop(&wall().ratio),
        AlignType::Start => wall().get_current_geometry().align_start(w, h),
        AlignType::Center => wall().get_current_geometry().align_center(w, h),
        AlignType::End => wall().get_current_geometry().align_end(w, h),
    };

    rsx! {
        PreviewableButton {
            wall,
            geom: geom.clone(),
            class,
            active: current_geom == geom,
            onclick: move |evt: MouseEvent| {
                wall.with_mut(|wall| {
                    // holding shift performs for align for all crops in the same direction
                    if evt.modifiers().shift() {
                        for (ratio, geom) in &mut wall.current.geometries {
                            // same direction
                            if geom.h == current_geom.h {
                                *geom = match align_type {
                                    AlignType::Source => wall.source.get_geometry(ratio),
                                    AlignType::Default => {
                                        let cropper = Cropper::new(&wall.current.faces, w, h);
                                        cropper.crop(ratio)
                                    },
                                    AlignType::Start => geom.align_start(w, h),
                                    AlignType::Center => geom.align_center(w, h),
                                    AlignType::End => geom.align_end(w, h),
                                }
                            }
                        }
                    } else {
                        wall.set_current_geometry(&geom);
                    }
                });
            },
            {children}
        }
    }
}

#[component]
pub fn AlignButtons(wall: Signal<Wall>, class: Option<String>) -> Element {
    let Wall { current: info, .. } = wall();
    let geom = wall().get_current_geometry();
    let dir = info.direction(&geom);

    rsx! {
            span {
                class: "isolate rounded-md shadow-sm",
                AlignButton {
                    wall,
                    class: "text-sm rounded-l-md",
                    align_type: AlignType::Source,
                    "Source"
                }
                AlignButton {
                    wall,
                    class: "text-sm rounded-r-md",
                    align_type: AlignType::Default,
                    "Default"
                }
            }

            span {
                class: format!("isolate rounded-md shadow-sm {}", class.unwrap_or_default()),
                AlignButton {
                    wall,
                    class: "text-sm rounded-l-md",
                    align_type: AlignType::Start,
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignLeft }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignTop }
                    }
                }
                AlignButton {
                    wall,
                    class: "text-sm -ml-px",
                    align_type: AlignType::Center,
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignCenter }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignCenter }
                    }
               }
                AlignButton {
                    wall,
                    class: "text-sm rounded-r-md",
                    align_type: AlignType::End,
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignRight }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignBottom }
                    }
                }
            }
    }
}
