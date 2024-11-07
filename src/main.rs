#![allow(non_snake_case)]
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use dioxus::desktop::Config;
use dioxus::prelude::*;
use screens::app::App;
use wallfacer::cli::{Commands, ShellCompletion, WallfacerArgs};

pub mod add_resolution;
pub mod add_wallpapers;
pub mod components;
pub mod screens;
pub mod state;
pub mod trimmer;

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
        Some(Commands::Add(args)) => add_wallpapers::main(&args),
        Some(Commands::AddResolution(args)) => add_resolution::main(&args),
        Some(Commands::Trim(args)) => trimmer::main(&args),
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
