#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{components::button::PreviewableButton, state::Wall};
use wallfacer::aspect_ratio::AspectRatio;

pub fn change_ratio(wall: &mut Signal<Wall>, ratio: &AspectRatio) {
    wall.with_mut(|wall| {
        wall.ratio = ratio.clone();
    });
}

#[component]
pub fn RatioButtons(wall: Signal<Wall>, class: Option<String>) -> Element {
    let ratios = wall().ratios;

    let len = ratios.len();

    let buttons = ratios.into_iter().enumerate().map(|(i, res)| {
        let cls = if i == 0 {
            "rounded-l-md"
        } else if i == len - 1 {
            "rounded-r-md"
        } else {
            "-ml-px"
        };

        let current_geom = wall().current.get_geometry(&res.resolution);
        let dirty_marker = if current_geom == wall().source.get_geometry(&res.resolution) {
            " "
        } else {
            "*"
        };

        rsx! {
            PreviewableButton {
                class: "text-sm {cls}",
                geom: current_geom,
                wall,
                title: res.description,
                active: wall().ratio == res.resolution,
                onclick: move |_| {
                    change_ratio(&mut wall, &res.resolution);
                },
                span {
                    class: "whitespace-pre",
                    "  {res.name} {dirty_marker}"
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
