#![allow(non_snake_case)]
use dioxus::prelude::*;
use itertools::Itertools;
use wallpaper_ui::wallpapers::WallInfo;

use crate::{app_state::UiState, buttons::Button};

#[derive(Clone, PartialEq, Props)]
pub struct CandidatesProps {
    class: Option<String>,
    info: Signal<WallInfo>,
    ui: Signal<UiState>,
}

pub fn Candidates(mut props: CandidatesProps) -> Element {
    let mut selected_candidate = use_signal(|| None);
    let current_ratio = (props.ui)().ratio;

    let info = (props.info)();
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
                let btn_cls = if (selected_candidate)() == Some(i) {
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
                                props.info.with_mut(|info| {
                                    info.set_geometry(&current_ratio, &geom);
                                });
                                selected_candidate.set(Some(i));
                            }
                        },
                    }
                }
            })}
        }
    }
}
