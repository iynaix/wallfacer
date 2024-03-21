#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::AspectRatio,
    wallpapers::{WallInfo, Wallpapers},
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

        rsx! {
            Button {
                class: "text-sm {cls}",
                text: format!("{}x{}", res.0, res.1),
                onclick: move |_| {
                    props.current_ratio.with_mut(|ratio| *ratio = res.clone());
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

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Props)]
pub struct ToolbarProps {
    wall_info: Signal<WallInfo>,
    current_ratio: Signal<AspectRatio>,
    show_faces: Signal<bool>,
    show_filelist: Signal<bool>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn Toolbar(mut props: ToolbarProps) -> Element {
    let info = (props.wall_info)();

    rsx! {
        div {
            class:"flex flex-row justify-between",

            // resolution selector on left
            ResolutionSelector {
                current_ratio: props.current_ratio,
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
                },

                Button {
                    class: "ml-8 content-end rounded-md text-sm",
                    text: "Save",
                    onclick: move |_| {
                        let mut wallpapers = Wallpapers::new();
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
