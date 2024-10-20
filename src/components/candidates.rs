#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallfacer::geometry::Geometry;

use crate::components::{button::Button, use_wallpapers};

#[component]
pub fn Candidates(class: Option<String>) -> Element {
    let mut wallpapers = use_wallpapers();
    let mut prev_geometry = use_signal(Geometry::default);

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
                                wallpapers.with_mut(|wallpapers| {
                                    prev_geometry.set(wallpapers.get_geometry());
                                    wallpapers.set_geometry(&geom);
                                });
                            }
                        },
                        onmouseleave: move |_| {
                            wallpapers.with_mut(|wallpapers| {
                                wallpapers.set_geometry(&prev_geometry());
                            });
                        },
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
