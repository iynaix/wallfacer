#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::AspectRatio,
    geometry::Geometry,
    wallpapers::{WallInfo, WallpapersCsv},
};

use crate::{align_group::AlignGroup, buttons::Button, switch::Switch};

const RESOLUTIONS: [AspectRatio; 5] = [
    AspectRatio(1440, 2560),
    AspectRatio(2256, 1504),
    AspectRatio(3440, 1440),
    AspectRatio(1920, 1080),
    AspectRatio(1, 1),
];

#[derive(Clone, PartialEq, Props)]
pub struct ResolutionSelectorProps {
    class: Option<String>,
    current_ratio: Signal<AspectRatio>,
    preview_geometry: Signal<Option<Geometry>>,
}

fn ResolutionSelector(mut props: ResolutionSelectorProps) -> Element {
    let buttons = RESOLUTIONS.iter().enumerate().map(|(i, res)| {
        let cls = if i == 0 {
            "rounded-l-md"
        } else if i == RESOLUTIONS.len() - 1 {
            "rounded-r-md"
        } else {
            "-ml-px"
        };

        let active = (props.current_ratio)() == *res;

        rsx! {
            Button {
                class: "text-sm {cls}",
                active: active,
                text: format!("{}x{}", res.0, res.1),
                onclick: move |_| {
                    props.current_ratio.set(res.clone());
                    props.preview_geometry.set(None);
                },
            }
        }
    });

    rsx! {
        span {
            class: "isolate inline-flex rounded-md shadow-sm",
            {buttons}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarProps {
    wall_info: Signal<WallInfo>,
    current_ratio: Signal<AspectRatio>,
    show_faces: Signal<bool>,
    show_filelist: Signal<bool>,
    manual_mode: Signal<bool>,
    preview_geometry: Signal<Option<Geometry>>,
}

pub fn Toolbar(mut props: ToolbarProps) -> Element {
    let info = (props.wall_info)();

    rsx! {
        div {
            class:"flex flex-row justify-between",

            // resolution selector on left
            ResolutionSelector {
                current_ratio: props.current_ratio,
                preview_geometry: props.preview_geometry,
            },

            // rest of toolbar
            div{
                class: "flex justify-end",

                Switch {
                    label: "Faces ({info.faces.len()})",
                    checked: props.show_faces,
                },

                AlignGroup {
                    class: "ml-16 content-end",
                    wall_info: props.wall_info,
                    current_ratio: (props.current_ratio)(),
                    manual_mode: props.manual_mode,
                },

                Button {
                    class: "ml-8 content-end rounded-md text-sm",
                    text: "Save",
                    onclick: move |_| {
                        let mut wallpapers = WallpapersCsv::new();
                        wallpapers.insert(info.filename.clone(), info.clone());
                        wallpapers.save();
                    },
                },

                Button {
                    class: "ml-8 content-end rounded-md text-sm",
                    text: "Tree",
                    onclick: move |_| {
                        props.show_filelist.set(!(props.show_filelist)());
                    },
                }
            }
        }
    }
}
