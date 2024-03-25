#![allow(non_snake_case)]
use dioxus::prelude::*;
use itertools::Itertools;

use crate::{
    app_state::{PreviewMode, UiState, Wallpapers},
    buttons::Button,
};

#[derive(Clone, PartialEq, Props)]
pub struct CandidatesProps {
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
}

pub fn Candidates(mut props: CandidatesProps) -> Element {
    if (props.ui)().preview_mode == PreviewMode::Manual {
        return None;
    }

    let walls = (props.wallpapers)();
    let current_geom = walls.get_geometry();

    if walls.current.faces.len() <= 1 {
        return None;
    }

    let candidates_geom: Vec<_> = walls
        .crop_candidates()
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
                                    ui.preview_mode = PreviewMode::Candidate(Some(geom.clone()));
                                });
                            }
                        },
                        onmouseleave: move |_| {
                            props.ui.with_mut(|ui| {
                                ui.preview_mode = PreviewMode::Candidate(None);
                            });
                        },
                        onclick: {
                            move |_| {
                                props.wallpapers.with_mut(|wallpapers| {
                                    wallpapers.set_geometry(&geom);
                                });
                                props.ui.with_mut(|ui| {
                                    ui.preview_mode = PreviewMode::None;
                                });
                            }
                        },
                    }
                }
            })}
        }
    }
}
