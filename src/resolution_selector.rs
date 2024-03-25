#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::cropper::AspectRatio;

use crate::{app_state::UiState, buttons::Button};

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
    ui: Signal<UiState>,
}

pub fn ResolutionSelector(mut props: ResolutionSelectorProps) -> Element {
    let buttons = RESOLUTIONS.iter().enumerate().map(|(i, res)| {
        let cls = if i == 0 {
            "rounded-l-md"
        } else if i == RESOLUTIONS.len() - 1 {
            "rounded-r-md"
        } else {
            "-ml-px"
        };

        let active = (props.ui)().ratio == *res;

        rsx! {
            Button {
                class: "text-sm {cls}",
                active: active,
                text: format!("{}x{}", res.0, res.1),
                onclick: move |_| {
                    props.ui.with_mut(|ui| {
                        ui.ratio = res.clone();
                    });
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
