#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{components::button::PreviewableButton, state::Wall};

pub fn prev_candidate(wall: &mut Signal<Wall>) {
    let candidates_geom = wall().candidate_geometries();
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

    let current_geom = wall().get_geometry();
    let idx = candidates_geom
        .binary_search(&current_geom)
        .map_or_else(decrement, decrement);

    wall.with_mut(|wall| {
        wall.set_geometry(&candidates_geom[idx]);
    });
}

pub fn next_candidate(wall: &mut Signal<Wall>) {
    let candidates_geom = wall().candidate_geometries();
    if candidates_geom.len() <= 1 {
        return;
    }

    let current_geom = wall().get_geometry();
    let idx = candidates_geom
        .binary_search(&current_geom)
        .map_or_else(|inserted| inserted, |found| found + 1)
        // handle wraparound
        % candidates_geom.len();

    wall.with_mut(|wall| {
        wall.set_geometry(&candidates_geom[idx]);
    });
}

#[component]
pub fn Candidates(wall: Signal<Wall>, class: Option<String>) -> Element {
    let info = wall().current;
    let current_geom = wall().get_geometry();

    if info.faces.len() <= 1 {
        return None;
    }

    let candidates_geom = wall().candidate_geometries();
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
                            wall.with_mut(|wall| {
                                wall.set_geometry(&geom);
                            });
                        },
                        {(i + 1).to_string()}
                    }
                }
            })}
        }
    }
}
