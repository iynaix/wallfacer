#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    app_state::{PreviewMode, UiState, Wallpapers},
    components::button::Button,
};

#[component]
pub fn Candidates(
    class: Option<String>,
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
) -> Element {
    if ui().preview_mode == PreviewMode::Pan {
        return None;
    }

    let walls = wallpapers();
    let current_geom = walls.get_geometry();

    if walls.current.faces.len() <= 1 {
        return None;
    }

    let candidates_geom = walls.candidate_geometries();
    if candidates_geom.len() <= 1 {
        return None;
    }

    rsx! {
        div {
            class: "flex",
            class: class.unwrap_or_default(),

            {candidates_geom.into_iter().enumerate().map(|(i, geom)| {
                let btn_cls = if geom == current_geom {
                    "!bg-indigo-600"
                } else {
                    ""
                };

                rsx! {
                    Button {
                        class: "flex-1 justify-center text-sm {btn_cls}",
                        onmouseenter: {
                            let geom = geom.clone();
                            move |_| {
                                ui.with_mut(|ui| {
                                    ui.preview_mode = PreviewMode::Candidate(Some(geom.clone()));
                                });
                            }
                        },
                        onmouseleave: move |_| {
                            ui.with_mut(|ui| {
                                ui.preview_mode = PreviewMode::Candidate(None);
                            });
                        },
                        onclick: move |_| {
                            wallpapers.with_mut(|wallpapers| {
                                wallpapers.set_geometry(&geom);
                            });
                            ui.with_mut(|ui| {
                                ui.preview_mode = PreviewMode::Candidate(None);
                            });
                        },
                        {(i + 1).to_string()}
                    }
                }
            })}
        }
    }
}
