#![allow(non_snake_case)]
use dioxus::prelude::*;
use itertools::Itertools;
use std::time::Instant;
use wallfacer::geometry::Geometry;

use crate::{
    components::{
        align_buttons::AlignButtons,
        app_header::{next_image, prev_image},
        candidates::{next_candidate, prev_candidate, Candidates},
        preview::Previewer,
        ratio_buttons::{change_ratio, RatioButtons},
    },
    state::{UiState, Wall, Wallpapers},
};

pub fn handle_arrow_keys_keyup(_arrow_key: &Key, ui: &mut Signal<UiState>) {
    ui.with_mut(|ui| {
        ui.arrow_key_start = None;
    });
}

pub fn handle_arrows_keydown(arrow_key: &Key, wall: &mut Signal<Wall>, ui: &mut Signal<UiState>) {
    let start_time_ms = ui()
        .arrow_key_start
        .map_or(0, |start_time| start_time.elapsed().as_millis());
    let velocity = (start_time_ms as f64 / 100.0).mul_add(4.0, 0.0).max(20.0);
    // minimum move distance is 1px
    let delta = (velocity * start_time_ms as f64 / 1000.0).max(1.0);

    match arrow_key {
        Key::ArrowLeft | Key::ArrowUp => {
            if start_time_ms == 0 {
                ui.with_mut(|ui| {
                    ui.arrow_key_start = Some(Instant::now());
                });
            }

            let new_geom = wall().move_geometry_by(-delta);
            wall.with_mut(|wall| {
                wall.set_geometry(&new_geom);
            });
        }

        Key::ArrowRight | Key::ArrowDown => {
            if start_time_ms == 0 {
                ui.with_mut(|ui| {
                    ui.arrow_key_start = Some(Instant::now());
                });
            }

            let new_geom = wall().move_geometry_by(delta);
            wall.with_mut(|wall| {
                wall.set_geometry(&new_geom);
            });
        }

        _ => {}
    }
}

pub fn handle_editor_shortcuts(
    evt: &Event<KeyboardData>,
    wall: &mut Signal<Wall>,
    wallpapers: &mut Signal<Wallpapers>,
    ui: &mut Signal<UiState>,
) {
    let Wall {
        current,
        source,
        ratio,
        ..
    } = wall();
    let geom = wall().get_geometry();
    let mut set_geom = |geom: Geometry| wall.with_mut(|wall| wall.set_geometry(&geom));

    match evt.key() {
        Key::Character(shortcut) => {
            let shortcut = shortcut.as_str();

            match shortcut {
                "f" => {
                    ui.with_mut(|ui| {
                        ui.show_faces = !ui.show_faces;
                    });
                }

                "h" => {
                    prev_image(wallpapers);
                }

                "l" => {
                    next_image(wallpapers);
                }

                "0" => {
                    set_geom(geom.align_start(current.width, current.height));
                }

                "m" => {
                    set_geom(geom.align_center(current.width, current.height));
                }

                "p" => {
                    prev_candidate(wall);
                }

                "n" => {
                    next_candidate(wall);
                }

                "$" => {
                    set_geom(geom.align_end(current.width, current.height));
                }

                "u" => {
                    set_geom(source.get_geometry(&ratio));
                }

                "d" => {
                    set_geom(current.cropper().crop(&ratio));
                }

                // tab through ratios
                "t" => {
                    let ratios = wall()
                        .ratios
                        .into_iter()
                        .map(|r| r.resolution)
                        .collect_vec();

                    if let Some(pos) = ratios.iter().position(|r| *r == ratio) {
                        let next = (pos + 1) % ratios.len();
                        change_ratio(wall, &ratios[next]);
                    }
                }
                _ => {}
            }
        }

        key => handle_arrows_keydown(&key, wall, ui),
    };
}

#[component]
pub fn Editor(wall: Signal<Wall>) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-4 w-full h-full",

            div {
                class:"flex flex-row justify-between",
                RatioButtons { wall },

                div{
                    class: "flex justify-end",
                    AlignButtons { wall },
                }
            }

            Previewer { wall }

            Candidates { wall }
        }
    }
}
