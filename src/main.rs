#![allow(non_snake_case)]
use clap::Parser;
use components::{
    align_selector::{set_align, toggle_pan},
    app_header::{next_image, prev_image, save_image},
};
use dioxus::desktop::Config;
use dioxus::prelude::*;
use wallpaper_ui::config::WallpaperConfig;

pub mod app_state;
pub mod cli;
pub mod components;

use crate::{
    app_state::{UiState, Wallpapers},
    components::{app_header::AppHeader, editor::Editor, filelist::FileList, wallust::Wallust},
};

fn main() {
    let args = cli::WallpaperUIArgs::parse();
    if args.version {
        println!("wallpaper-ui {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    // use a custom index.html to set the height of body to the full height of the window
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_background_color((30, 30, 46, 255))
                .with_menu(None)
                // disable on release builds
                .with_disable_context_menu(!cfg!(debug_assertions))
                .with_custom_index(
                    r#"<!DOCTYPE html>
<html>
    <head>
        <title>Dioxus app</title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <link rel="stylesheet" href="public/tailwind.css">
    </head>
    <body>
        <div id="main" style="height: 100vh;"></div>
    </body>
</html>"#
                        .to_string(),
                ),
        )
        .launch(App);
}

// #[allow(clippy::too_many_lines)]
fn handle_shortcuts(
    event: &Event<KeyboardData>,
    wallpapers: &mut Signal<Wallpapers>,
    ui: &mut Signal<UiState>,
) {
    let walls = wallpapers();

    // handle keys with modifiers first
    if event.modifiers().ctrl() {
        if let Key::Character(char) = event.key() {
            match char.as_str() {
                "f" => {
                    ui.with_mut(|ui| {
                        ui.show_filelist = !ui.show_filelist;
                    });
                }
                "s" => {
                    if !walls.files.is_empty() {
                        save_image(wallpapers, ui);
                    }
                }
                _ => {}
            }
        }

        return;
    }

    match event.key() {
        Key::Character(char) => {
            match char.as_str() {
                "/" => {
                    ui.with_mut(|ui| {
                        ui.show_filelist = !ui.show_filelist;
                    });
                }

                "f" => {
                    ui.with_mut(|ui| {
                        ui.show_faces = !ui.show_faces;
                    });
                }

                "h" => {
                    if !ui().show_filelist {
                        prev_image(wallpapers, ui);
                    }
                }

                "l" => {
                    if !ui().show_filelist {
                        next_image(wallpapers, ui);
                    }
                }

                // alignment
                "0" => {
                    if !ui().show_filelist {
                        set_align(
                            &walls
                                .get_geometry()
                                .align_start(walls.current.width, walls.current.height),
                            wallpapers,
                            ui,
                        );
                    }
                }

                "m" => {
                    if !ui().show_filelist {
                        set_align(
                            &walls
                                .get_geometry()
                                .align_center(walls.current.width, walls.current.height),
                            wallpapers,
                            ui,
                        );
                    }
                }

                "$" => {
                    if !ui().show_filelist {
                        set_align(
                            &walls
                                .get_geometry()
                                .align_end(walls.current.width, walls.current.height),
                            wallpapers,
                            ui,
                        );
                    }
                }

                "u" => {
                    if !ui().show_filelist {
                        set_align(&walls.source.get_geometry(&walls.ratio), wallpapers, ui);
                    }
                }

                "d" => {
                    if !ui().show_filelist {
                        set_align(&walls.current.cropper().crop(&walls.ratio), wallpapers, ui);
                    }
                }

                // palette
                "p" => {
                    ui.with_mut(|ui| {
                        ui.show_palette = !ui.show_palette;
                    });
                }

                // panning
                " " => {
                    if !ui().show_filelist {
                        toggle_pan(ui);
                    }
                }

                // tab through ratios
                "t" => {
                    if !ui().show_filelist {
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
                }

                _ => {}
            }
        }

        Key::ArrowLeft => {
            if !ui().show_filelist {
                prev_image(wallpapers, ui);
            }
        }

        Key::ArrowRight => {
            if !ui().show_filelist {
                next_image(wallpapers, ui);
            }
        }

        _ => {}
    };
}

// define a component that renders a div with the text "Hello, world!"
fn App() -> Element {
    let config = WallpaperConfig::new();
    let mut wallpapers = use_signal(|| Wallpapers::from_args(&config.wallpapers_path));
    let mut ui = use_signal(|| UiState {
        show_faces: config.show_faces,
        ..UiState::default()
    });
    let has_files = !wallpapers().files.is_empty();

    if !has_files {
        return rsx! {
            main {
                class: "dark flex items-center h-full justify-center bg-base overflow-hidden",
                div {
                    h1 { class: "mt-4 text-4xl font-bold tracking-tight text-text text-center h-full",
                        "No more wallpapers to process! 🎉"
                    }
                }
            }
        };
    }

    rsx! {
        main {
            class: "dark flex flex-col h-full bg-base overflow-hidden",
            tabindex: 0,
            onkeydown: move |event| {
                handle_shortcuts(&event, &mut wallpapers, &mut ui);
            },

            AppHeader { wallpapers, ui }

            div {
                class: "flex p-4 gap-4",

                if (ui)().show_filelist {
                    FileList { wallpapers, ui }
                } else if (ui)().show_palette {
                    Wallust { wallpapers }
                } else {
                    Editor { wallpapers, ui, wallpapers_path: config.wallpapers_path }
                }
            }
        }
    }
}
