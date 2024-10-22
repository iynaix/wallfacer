#![allow(non_snake_case)]
use dioxus::prelude::*;
use std::time::Instant;

use crate::{
    components::{
        align_buttons::{set_align, AlignButtons},
        app_header::{next_image, prev_image},
        candidates::{next_candidate, prev_candidate, Candidates},
        preview::Previewer,
        ratio_buttons::{change_ratio, RatioButtons},
    },
    state::{UiState, Wallpapers},
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

            let new_geom = wallpapers().move_geometry_by(-delta);
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

            let new_geom = wallpapers().move_geometry_by(delta);
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

                "p" => {
                    prev_candidate();
                }

                "n" => {
                    next_candidate();
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
pub fn Editor() -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-4 w-full h-full",

            div {
                class:"flex flex-row justify-between",
                RatioButtons { },

                div{
                    class: "flex justify-end",
                    AlignButtons { },
                }
            }

            Previewer { }

            Candidates { }
        }
    }
}
