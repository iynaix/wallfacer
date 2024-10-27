#![allow(non_snake_case)]
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use dioxus::desktop::Config;
use dioxus::prelude::*;
use screens::app::App;
use std::path::PathBuf;

pub mod add_resolution;
pub mod add_wallpapers;
pub mod components;
pub mod screens;
pub mod state;

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

    rexiv2::register_xmp_namespace("http://example.com/wallfacer", "wallfacer")
        .expect("could not register wallfacer namespace");

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
