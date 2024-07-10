#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::icons::{
    md_action_icons::MdPanTool,
    md_editor_icons::{
        MdFormatAlignCenter, MdFormatAlignLeft, MdFormatAlignRight, MdVerticalAlignBottom,
        MdVerticalAlignCenter, MdVerticalAlignTop,
    },
};
use dioxus_free_icons::Icon;
use wallpaper_ui::{cropper::Direction, geometry::Geometry};

use crate::{
    app_state::PreviewMode,
    components::{button::Button, use_ui, use_wallpapers},
};

pub fn set_align(geom: &Geometry) {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    wallpapers.with_mut(|wallpapers| {
        wallpapers.set_geometry(geom);
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
    });
}

#[component]
fn AlignButton(class: String, geom: Geometry, children: Element) -> Element {
    let current_geom = use_wallpapers()().get_geometry();

    rsx! {
        Button {
            class,
            active: current_geom == geom,
            onclick: move |_| {
                set_align(&geom);
            },
            {children}
        }
    }
}

pub fn toggle_pan() {
    use_ui().with_mut(|ui| {
        ui.preview_mode = if matches!(&ui.preview_mode, PreviewMode::Pan) {
            PreviewMode::Candidate(None)
        } else {
            PreviewMode::Pan
        }
    });
}

#[component]
pub fn AlignSelector(class: Option<String>) -> Element {
    let ui = use_ui();

    let walls = &use_wallpapers()();
    let info = &walls.current;
    let ratio = &walls.ratio;
    let align = ui().preview_mode;
    let geom = &walls.get_geometry();
    let dir = info.direction(geom);

    rsx! {
        div { class: "flex gap-x-6",
            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                AlignButton {
                    class: "text-sm rounded-l-md",
                    geom: walls.source.get_geometry(ratio),
                    "Source"
                }
                AlignButton {
                    class: "text-sm rounded-r-md",
                    geom: info.cropper().crop(ratio),
                    "Default"
                }
            }

            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                class: class.unwrap_or_default(),
                AlignButton {
                    class: "text-sm rounded-l-md",
                    geom: geom.align_start(info.width, info.height),
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignLeft }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignTop }
                    }
                }
                AlignButton {
                    class: "text-sm -ml-px",
                    geom: geom.align_center(info.width, info.height),
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignCenter }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignCenter }
                    }
               }
                AlignButton {
                    class: "text-sm rounded-r-md",
                    geom: geom.align_end(info.width, info.height),
                    if dir == Direction::X {
                        Icon { fill: "white", icon:  MdFormatAlignRight }
                    } else {
                        Icon { fill: "white", icon: MdVerticalAlignBottom }
                    }
                }
            }

            span {
                class: "isolate inline-flex rounded-md shadow-sm",
                Button {
                    class: "text-sm rounded-md",
                    active: align == PreviewMode::Pan,
                    onclick: move |_| {
                        toggle_pan();
                    },
                    Icon { fill: "white", icon: MdPanTool }
                }
            }
        }
    }
}
