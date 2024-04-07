#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    app_state::{UiState, Wallpapers},
    buttons::Button,
};

#[component]
pub fn RatioSelector(
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
) -> Element {
    let ratios: Vec<_> = wallpapers()
        .current
        .sorted_geometries()
        .map(|(ratio, _)| ratio.clone())
        .collect();
    let len = ratios.len();

    let buttons = ratios.into_iter().enumerate().map(|(i, res)| {
        let cls = if i == 0 {
            "rounded-l-md"
        } else if i == len - 1 {
            "rounded-r-md"
        } else {
            "-ml-px"
        };

        let walls = wallpapers();
        let is_active = walls.ratio == res;
        let dirty_marker = if walls.current.get_geometry(&res) == walls.source.get_geometry(&res) {
            ""
        } else {
            " *"
        };

        let btn_text = format!("{}x{}{}", &res.0, &res.1, dirty_marker);

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
