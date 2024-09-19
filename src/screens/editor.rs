#![allow(non_snake_case)]
use dioxus::prelude::*;
use std::{path::PathBuf, time::Instant};

use crate::{
    app_state::{PreviewMode, UiState, Wallpapers},
    components::{
        align_selector::{set_align, toggle_pan, AlignSelector},
        app_header::{next_image, prev_image},
        candidates::Candidates,
        preview::Previewer,
        ratio_selector::{change_ratio, RatioSelector},
    },
};

pub fn handle_arrow_keys_keyup(_arrow_key: &Key, ui: &mut Signal<UiState>) {
    ui.with_mut(|ui| {
        ui.arrow_key_start = None;
    });
}

pub fn handle_arrows_keydown(
    arrow_key: &Key,
    wallpapers: &mut Signal<Wallpapers>,
    ui: &mut Signal<UiState>,
) {
    let walls = wallpapers();
    let current_geom = walls.get_geometry();
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

            let new_geom = match ui().preview_mode {
                PreviewMode::Candidate(_) => {
                    let candidates_geom = walls.candidate_geometries();
                    candidates_geom
                        .iter()
                        .position(|geom| *geom == current_geom)
                        .map_or_else(
                            || candidates_geom[0].clone(),
                            |pos| {
                                // has candidates
                                if candidates_geom.len() > 1 {
                                    let prev = if pos == 0 {
                                        candidates_geom.len() - 1
                                    } else {
                                        pos - 1
                                    };
                                    candidates_geom[prev].clone()
                                } else {
                                    // no candidates, start move by delta
                                    ui.with_mut(|ui| {
                                        ui.preview_mode = PreviewMode::Pan;
                                    });
                                    wallpapers().move_geometry_by(-delta)
                                }
                            },
                        )
                }
                PreviewMode::Pan => wallpapers().move_geometry_by(-delta),
            };

            wallpapers.with_mut(|wallpapers| {
                wallpapers.set_geometry(&new_geom);
            });
        }

        Key::ArrowRight | Key::ArrowDown => {
            if start_time_ms == 0 {
                ui.with_mut(|ui| {
                    ui.arrow_key_start = Some(Instant::now());
                });
            }

            let new_geom = match ui().preview_mode {
                PreviewMode::Candidate(_) => {
                    let candidates_geom = walls.candidate_geometries();
                    candidates_geom
                        .iter()
                        .position(|geom| *geom == current_geom)
                        .map_or_else(
                            || candidates_geom[0].clone(),
                            |pos| {
                                // has candidates
                                if candidates_geom.len() > 1 {
                                    let next = (pos + 1) % candidates_geom.len();
                                    candidates_geom[next].clone()
                                } else {
                                    // no candidates, start move by delta
                                    ui.with_mut(|ui| {
                                        ui.preview_mode = PreviewMode::Pan;
                                    });
                                    wallpapers().move_geometry_by(delta)
                                }
                            },
                        )
                }
                PreviewMode::Pan => wallpapers().move_geometry_by(delta),
            };

            wallpapers.with_mut(|wallpapers| {
                wallpapers.set_geometry(&new_geom);
            });
        }

        _ => {}
    }
}

pub fn handle_editor_shortcuts(
    evt: &Event<KeyboardData>,
    wallpapers: &mut Signal<Wallpapers>,
    ui: &mut Signal<UiState>,
) {
    let walls = wallpapers();

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
                    prev_image();
                }

                "l" => {
                    next_image();
                }

                "0" => {
                    set_align(
                        &walls
                            .get_geometry()
                            .align_start(walls.current.width, walls.current.height),
                    );
                }

                "m" => {
                    set_align(
                        &walls
                            .get_geometry()
                            .align_center(walls.current.width, walls.current.height),
                    );
                }

                "$" => {
                    set_align(
                        &walls
                            .get_geometry()
                            .align_end(walls.current.width, walls.current.height),
                    );
                }

                "u" => {
                    set_align(&walls.source.get_geometry(&walls.ratio));
                }

                "d" => {
                    set_align(&walls.current.cropper().crop(&walls.ratio));
                }

                " " => {
                    toggle_pan();
                }

                // tab through ratios
                "t" => {
                    let ratios = walls
                        .image_ratios()
                        .into_iter()
                        .map(|(_, r)| r)
                        .collect::<Vec<_>>();

                    if let Some(pos) = ratios.iter().position(|r| *r == walls.ratio) {
                        let next = (pos + 1) % ratios.len();
                        change_ratio(&ratios[next]);
                    }
                }
                _ => {}
            }
        }

        key => handle_arrows_keydown(&key, wallpapers, ui),
    };
}

#[component]
pub fn Editor(wallpapers_path: PathBuf) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-4 w-full h-full",

            div {
                class:"flex flex-row justify-between",
                RatioSelector { },

                div{
                    class: "flex justify-end",
                    AlignSelector { },
                }
            }

            Previewer { wallpapers_path }

            Candidates { }
        }
    }
}
