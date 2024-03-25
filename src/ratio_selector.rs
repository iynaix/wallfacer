#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::cropper::AspectRatio;

use crate::{
    app_state::{UiState, Wallpapers},
    buttons::Button,
};

const RATIOS: [AspectRatio; 5] = [
    AspectRatio(1440, 2560),
    AspectRatio(2256, 1504),
    AspectRatio(3440, 1440),
    AspectRatio(1920, 1080),
    AspectRatio(1, 1),
];

#[component]
pub fn RatioSelector(
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
) -> Element {
    let buttons = RATIOS.iter().enumerate().map(|(i, res)| {
        let cls = if i == 0 {
            "rounded-l-md"
        } else if i == RATIOS.len() - 1 {
            "rounded-r-md"
        } else {
            "-ml-px"
        };

        let walls = wallpapers();
        let is_active = walls.ratio == *res;
        let dirty_marker = if walls.current.get_geometry(res) == walls.source.get_geometry(res) {
            ""
        } else {
            " *"
        };

        rsx! {
            Button {
                class: "text-sm {cls}",
                active: is_active,
                text: format!("{}x{}{}", res.0, res.1, dirty_marker),
                onclick: move |_| {
                    wallpapers.with_mut(|wallpapers| {
                        wallpapers.ratio = res.clone();
                    });
                }
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
