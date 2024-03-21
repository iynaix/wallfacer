#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::{AspectRatio, Direction, Geometry},
    wallpapers::{WallInfo, Wallpapers},
};

// urls are relative to your Cargo.toml file
const _TAILWIND_URL: &str = manganis::mg!(file("./public/tailwind.css"));

pub mod buttons;
pub mod preview;
pub mod switch;

use crate::{buttons::Button, preview::Previewer, switch::Switch};

const RESOLUTIONS: [AspectRatio; 5] = [
    AspectRatio(1440, 2560),
    AspectRatio(2256, 1504),
    AspectRatio(3440, 1440),
    AspectRatio(1920, 1080),
    AspectRatio(1, 1),
];

fn main() {
    launch(App);
}

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
            for button in buttons {
                {button}
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct CropAlignSelectorProps {
    class: Option<String>,
    wall_info: Signal<WallInfo>,
    current_ratio: AspectRatio,
}
fn CropAlignSelector(mut props: CropAlignSelectorProps) -> Element {
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
                            info.set_geometry(&ratio, &info.cropper().crop(&ratio).geometry());
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
                class: "text-sm rounded-r-md",
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
        }
    }
}

// define a component that renders a div with the text "Hello, world!"
fn App() -> Element {
    let wall = "71124299_p0.png";
    // let wall = "107610529_p1.png";

    let mut wallpapers = Wallpapers::new();

    let show_faces = use_signal(|| false);
    let wall_info = use_signal(|| wallpapers[wall].clone());
    let current_ratio = use_signal(|| AspectRatio(1440, 2560));

    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        body {
            class: "dark bg-base",
            height: "100%",
            width: "100%",
            position: "absolute",
            flex: 1,

            div {
                class:"p-4 flex flex-row justify-between",

                ResolutionSelector {
                    current_ratio: current_ratio,
                },

                div{
                    class: "flex justify-end",

                    Switch {
                        label: "Faces",
                        checked: show_faces,
                     },

                    CropAlignSelector {
                        class: "ml-16 content-end",
                        wall_info: wall_info,
                        current_ratio: current_ratio(),
                    },

                    Button {
                        class: "ml-8 content-end rounded-md text-sm",
                        text: "Save",
                        onclick: move |_| {
                            wallpapers.insert(wall_info().filename, wall_info());
                            wallpapers.save();
                        },
                    },
                }
            }

            Previewer {
                wall_info: wall_info(),
                ratio: current_ratio(),
                show_faces: show_faces(),
            }
        }
    }
}
