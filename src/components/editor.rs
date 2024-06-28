#![allow(non_snake_case)]
use dioxus::prelude::*;
use std::path::PathBuf;

use crate::{
    app_state::{UiMode, UiState, Wallpapers},
    components::{
        align_selector::{set_align, toggle_pan, AlignSelector},
        app_header::{next_image, prev_image},
        candidates::Candidates,
        preview::Previewer,
        ratio_selector::RatioSelector,
    },
};

pub fn handle_editor_shortcuts(
    event: &Event<KeyboardData>,
    wallpapers: &mut Signal<Wallpapers>,
    ui: &mut Signal<UiState>,
) {
    let walls = wallpapers();

    match event.key() {
        Key::Character(shortcut) => {
            let shortcut = shortcut.as_str();

            match shortcut {
                "f" => {
                    ui.with_mut(|ui| {
                        ui.show_faces = !ui.show_faces;
                    });
                }

                "h" => {
                    prev_image(wallpapers, ui);
                }

                "l" => {
                    next_image(wallpapers, ui);
                }

                "0" => {
                    set_align(
                        &walls
                            .get_geometry()
                            .align_start(walls.current.width, walls.current.height),
                        wallpapers,
                        ui,
                    );
                }

                "m" => {
                    set_align(
                        &walls
                            .get_geometry()
                            .align_center(walls.current.width, walls.current.height),
                        wallpapers,
                        ui,
                    );
                }

                "$" => {
                    set_align(
                        &walls
                            .get_geometry()
                            .align_end(walls.current.width, walls.current.height),
                        wallpapers,
                        ui,
                    );
                }

                "u" => {
                    set_align(&walls.source.get_geometry(&walls.ratio), wallpapers, ui);
                }

                "d" => {
                    set_align(&walls.current.cropper().crop(&walls.ratio), wallpapers, ui);
                }

                " " => {
                    toggle_pan(ui);
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
                        wallpapers.with_mut(|wallpapers| {
                            wallpapers.ratio = ratios[next].clone();
                        });
                    }
                }
                _ => {}
            }
        }

        Key::ArrowLeft => {
            prev_image(wallpapers, ui);
        }

        Key::ArrowRight => {
            next_image(wallpapers, ui);
        }

        _ => {}
    };
}

#[component]
pub fn Editor(
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
    wallpapers_path: PathBuf,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-4 w-full h-full",

            div {
                class:"flex flex-row justify-between",
                RatioSelector { wallpapers, ui },

                div{
                    class: "flex justify-end",
                    AlignSelector { wallpapers, ui },
                }
            }

            Previewer { wallpapers, ui, wallpapers_path }

            Candidates { wallpapers, ui }
        }
    }
}
