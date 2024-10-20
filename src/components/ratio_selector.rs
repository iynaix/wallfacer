#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::components::{button::Button, use_wallpapers};
use wallfacer::aspect_ratio::AspectRatio;

pub fn change_ratio(ratio: &AspectRatio) {
    let mut wallpapers = use_wallpapers();

    wallpapers.with_mut(|wallpapers| {
        wallpapers.ratio = ratio.clone();
    });
}

#[component]
pub fn RatioSelector(class: Option<String>) -> Element {
    let walls = use_wallpapers()();
    let ratios = walls.image_ratios();

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
                onclick: move |_| {
                    change_ratio(&res);
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
