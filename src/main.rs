#![allow(non_snake_case)]
use app_state::PreviewMode;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use dioxus::desktop::Config;
use dioxus::prelude::*;
use std::path::PathBuf;

use wallfacer::config::WallpaperConfig;

pub mod add_resolution;
pub mod add_wallpapers;
pub mod app_state;
pub mod components;
pub mod screens;

use crate::{
    app_state::{UiMode, UiState, Wallpapers},
    components::{app_header::AppHeader, save_button::save_image},
    screens::{
        adding::Adding,
        editor::{handle_arrow_keys_keyup, handle_editor_shortcuts, Editor},
        filelist::FileList,
        palette::Palette,
    },
};

#[derive(ValueEnum, Debug, Clone)]
pub enum FacesFilter {
    Zero,
    None,
    One,
    Single,
    Many,
    Multiple,
    All,
}

// for generating shell completions
#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum ShellCompletion {
    Bash,
    Zsh,
    Fish,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        name = "add",
        about = "Adds wallpapers with upscaling and face detection"
    )]
    Add(add_wallpapers::AddWallpaperArgs),

    #[command(name = "resolution", about = "Adds a new resolution for cropping")]
    AddResolution(add_resolution::AddResolutionArgs),
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[command(
    name = "wallfacer",
    about = "A GUI for selecting wallpaper cropping regions for multiple monitor resolutions, based on anime face detection.",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct WallfacerArgs {
    #[arg(
        long,
        value_enum,
        help = "Type of shell completion to generate",
        hide = true,
        exclusive = true
    )]
    pub generate: Option<ShellCompletion>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(
        long,
        default_value = None,
        default_missing_value = "all",
        num_args = 0..=1,
        value_name = "RESOLUTIONS",
        help = "Only show wallpapers that use the default crops; either \"all\" or resolution(s) in the format \"1920x1080,1920x1200\""
    )]
    pub unmodified: Option<String>,

    #[arg(
        long,
        default_value = None,
        default_missing_value = "all",
        num_args = 0..=1,
        value_name = "RESOLUTIONS",
        help = "Only show wallpapers that don't use the default crops; either \"all\" or resolution(s) in the format \"1920x1080,1920x1200\""
    )]
    pub modified: Option<String>,

    #[arg(
        long,
        default_value = "all",
        default_missing_value = "all",
        value_parser = clap::value_parser!(FacesFilter),
        help = "Only show wallpapers that have a palette"
    )]
    pub faces: FacesFilter,

    #[arg(long, help = "Filters wallpapers by filename (case-insensitive)")]
    pub filter: Option<String>,

    #[arg(help = "Directories or images to be displayed", value_name = "PATHS")]
    pub paths: Option<Vec<PathBuf>>,
}

fn main() {
    let args = WallfacerArgs::parse();

    if let Some(comp) = args.generate {
        match comp {
            ShellCompletion::Bash => generate(
                Shell::Bash,
                &mut WallfacerArgs::command(),
                "wallfacer",
                &mut std::io::stdout(),
            ),
            ShellCompletion::Zsh => generate(
                Shell::Zsh,
                &mut WallfacerArgs::command(),
                "wallfacer",
                &mut std::io::stdout(),
            ),
            ShellCompletion::Fish => generate(
                Shell::Fish,
                &mut WallfacerArgs::command(),
                "wallfacer",
                &mut std::io::stdout(),
            ),
        }

        return;
    }

    match args.command {
        Some(Commands::Add(args)) => add_wallpapers::main(args),
        Some(Commands::AddResolution(args)) => add_resolution::main(args),
        _ => {
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
                                </html>"#.to_string(),
                        ),
                )
                .launch(App);
        }
    }
}

fn handle_shortcuts(
    evt: &Event<KeyboardData>,
    wallpapers: &mut Signal<Wallpapers>,
    ui: &mut Signal<UiState>,
) {
    match evt.key() {
        Key::Character(shortcut) => {
            let shortcut = shortcut.as_str();

            match shortcut {
                "/" => {
                    ui.with_mut(app_state::UiState::toggle_filelist);
                }

                // ctrl+f
                "f" => {
                    if evt.modifiers().ctrl() {
                        ui.with_mut(app_state::UiState::toggle_filelist);
                    }
                }

                // ctrl+s
                "s" => {
                    if evt.modifiers().ctrl() && !wallpapers().files.is_empty() {
                        save_image();
                    }
                }

                // palette
                "p" => {
                    if evt.modifiers().ctrl() && !wallpapers().files.is_empty() {
                        ui.with_mut(app_state::UiState::toggle_palette);
                    }
                }
                _ => {
                    if ui().mode == UiMode::Editor {
                        handle_editor_shortcuts(evt, wallpapers, ui);
                    }
                }
            }
        }
        _ => {
            if ui().mode == UiMode::Editor {
                handle_editor_shortcuts(evt, wallpapers, ui);
            }
        }
    };
}

fn App() -> Element {
    let config = use_context_provider(|| Signal::new(WallpaperConfig::new()));
    let mut wallpapers = use_context_provider(|| Signal::new(Wallpapers::from_args(&config())));
    let mut ui = use_context_provider(|| {
        let walls = wallpapers();
        let has_multiple_candidates =
            walls.current.cropper().crop_candidates(&walls.ratio).len() > 1;

        Signal::new(UiState {
            show_faces: config().show_faces,
            preview_mode: if has_multiple_candidates {
                PreviewMode::Candidate(None)
            } else {
                PreviewMode::Pan
            },
            ..UiState::default()
        })
    });

    if wallpapers().files.is_empty() {
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
            autofocus: true,
            onkeydown: move |evt| {
                handle_shortcuts(&evt, &mut wallpapers, &mut ui);
            },
            onkeyup: move |evt| {
                handle_arrow_keys_keyup(&evt.key(), &mut ui);
            },

            AppHeader { }

            div {
                class: "flex p-4 gap-4",

                if ui().mode == UiMode::FileList {
                    FileList { }
                } else if ui().mode == UiMode::Palette {
                    Palette { }
                } else if ui().mode == UiMode::Editor {
                    Editor { wallpapers_path: config().wallpapers_dir }
                } else if let UiMode::Adding(images) = ui().mode {
                    Adding { images }
                }
            }
        }
    }
}
