#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::components::{button::PreviewableButton, use_wallpapers};

pub fn prev_candidate() {
    let mut wallpapers = use_wallpapers();

    let walls = wallpapers();
    let current_geom = walls.get_geometry();

    let candidates_geom = walls.candidate_geometries();

    if candidates_geom.len() <= 1 {
        return;
    }

    let decrement = |n: usize| {
        // handle wraparound
        if n == 0 {
            candidates_geom.len() - 1
        } else {
            n - 1
        }
    };

    let idx = candidates_geom
        .binary_search(&current_geom)
        .map_or_else(decrement, decrement);

    wallpapers.with_mut(|wallpapers| {
        wallpapers.set_geometry(&candidates_geom[idx as usize]);
    });
}

pub fn next_candidate() {
    let mut wallpapers = use_wallpapers();

    let walls = wallpapers();
    let current_geom = walls.get_geometry();

    let candidates_geom = walls.candidate_geometries();

    if candidates_geom.len() <= 1 {
        return;
    }

    let idx = candidates_geom
        .binary_search(&current_geom)
        .map_or_else(|inserted| inserted, |found| found + 1)
        // handle wraparound
        % candidates_geom.len();

    wallpapers.with_mut(|wallpapers| {
        wallpapers.set_geometry(&candidates_geom[idx]);
    });
}

#[component]
pub fn Candidates(class: Option<String>) -> Element {
    let mut wallpapers = use_wallpapers();

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
                    PreviewableButton {
                        geom: geom.clone(),
                        class: "flex-1 justify-center text-sm {btn_cls}",
                        onclick: move |_| {
                            wallpapers.with_mut(|wallpapers| {
                                wallpapers.set_geometry(&geom);
                            });
                        },
                        {(i + 1).to_string()}
                    }
                }
            })}
        }
    }
}
