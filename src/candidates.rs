#![allow(non_snake_case)]
use dioxus::prelude::*;
use itertools::Itertools;

use crate::{
    app_state::{AlignMode, UiState, Wallpapers},
    buttons::Button,
};

#[derive(Clone, PartialEq, Props)]
pub struct CandidatesProps {
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
}

pub fn Candidates(mut props: CandidatesProps) -> Element {
    let current_ratio = (props.ui)().ratio;
    let info = (props.wallpapers)().current;
    let current_geom = info.get_geometry(&current_ratio);

    if info.faces.len() <= 1 {
        return None;
    }

    let candidates_geom: Vec<_> = info
        .cropper()
        .crop_candidates(&current_ratio)
        .into_iter()
        .unique()
        .enumerate()
        .collect();

    if candidates_geom.len() <= 1 {
        return None;
    }

    rsx! {
        div {
            class: "flex {props.class.unwrap_or_default()}",

            {candidates_geom.into_iter().map(|(i, geom)| {
                let btn_cls = if geom == current_geom {
                    "!bg-indigo-600"
                } else {
                    ""
                };

                rsx! {
                    Button {
                        class: "flex-1 justify-center text-sm {btn_cls}",
                        text: (i + 1).to_string(),
                        onmouseenter: {
                            let geom = geom.clone();
                            move |_| {
                                props.ui.with_mut(|ui| {
                                    ui.preview_geometry = Some(geom.clone());
                                });
                            }
                        },
                        onmouseleave: move |_| {
                            props.ui.with_mut(|ui| {
                                ui.preview_geometry = None;
                            });
                        },
                        onclick: {
                            let current_ratio = current_ratio.clone();
                            move |_| {
                                props.wallpapers.with_mut(|wallpapers| {
                                    wallpapers.current.set_geometry(&current_ratio, &geom);
                                });
                                props.ui.with_mut(|ui| {
                                    ui.preview_geometry = None;
                                    ui.align_mode = AlignMode::None;
                                });
                            }
                        },
                    }
                }
            })}
        }
    }
}
