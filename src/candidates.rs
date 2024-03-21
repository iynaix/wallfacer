#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::{AspectRatio, Geometry},
    wallpapers::WallInfo,
};

use crate::buttons::Button;

#[derive(Clone, PartialEq, Props)]
#[allow(clippy::module_name_repetitions)]
pub struct CandidatesProps {
    class: Option<String>,
    info: Signal<WallInfo>,
    current_ratio: AspectRatio,
    preview_geometry: Signal<Option<Geometry>>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn Candidates(mut props: CandidatesProps) -> Element {
    let info = (props.info)();
    if info.faces.len() <= 1 {
        return None;
    }

    let candidates = info.cropper().crop_candidates(&props.current_ratio);
    let candidates_btns = candidates
        .iter()
        .map(wallpaper_ui::wallpapers::Face::geometry)
        .enumerate()
        .map(|(i, geom)| {
            rsx! {
                Button {
                    class: "flex-1 justify-center text-sm",
                    text: (i + 1).to_string(),
                    onmouseenter: {
                        let geom = geom.clone();
                        move |_| {
                            props.preview_geometry.set(Some(geom.clone()));
                        }
                    },
                    onmouseleave: move |_| {
                        props.preview_geometry.set(None);
                    },
                    onclick: {
                        let ratio = props.current_ratio.clone();
                        move |_| {
                            props.info.with_mut(|info| {
                                info.set_geometry(&ratio, &geom);
                            });
                        }
                    },
                }
            }
        });

    rsx! {
        div {
            class: "flex {props.class.unwrap_or_default()}",

            {candidates_btns}
        }
    }
}
