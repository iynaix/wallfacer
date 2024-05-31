#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{
    app_state::{UiState, Wallpapers},
    button::Button,
};

#[component]
pub fn RatioSelector(
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
) -> Element {
    let walls = wallpapers();
    let ratios: Vec<_> = walls
        .resolutions
        .into_iter()
        .filter(|(_, ratio)| {
            // do not show resolution if aspect ratio of image is the same,
            // as there is only a single possible crop
            (f64::from(walls.current.width) / f64::from(walls.current.height) - f64::from(ratio))
                .abs()
                > f64::EPSILON
        })
        .collect();

    let len = ratios.len();

    let buttons = ratios.into_iter().enumerate().map(|(i, (res_name, res))| {
        let cls = if i == 0 {
            "rounded-l-md"
        } else if i == len - 1 {
            "rounded-r-md"
        } else {
            "-ml-px"
        };

        let is_active = walls.ratio == res;
        let dirty_marker = if walls.current.get_geometry(&res) == walls.source.get_geometry(&res) {
            ""
        } else {
            " *"
        };

        let btn_text = format!("{}{}", res_name, dirty_marker);

        rsx! {
            Button {
                class: "text-sm {cls}",
                active: is_active,
                onclick: move |_|{
                    wallpapers.with_mut(|wallpapers| {
                        wallpapers.ratio = res.clone();
                    });
                }
                {btn_text}
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
