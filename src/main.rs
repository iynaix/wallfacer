#![allow(non_snake_case)]
use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use dioxus::desktop::{Config, WindowBuilder};
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
    let all_args = WallfacerArgs::parse();

    rexiv2::register_xmp_namespace("http://example.com/wallfacer", "wallfacer")
        .expect("could not register wallfacer namespace");

    if let Some(comp) = all_args.generate {
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

    match all_args.command {
        Some(Commands::Add(args)) => add_wallpapers::main(all_args.config, &args),
        Some(Commands::AddResolution(args)) => add_resolution::main(all_args.config, &args),
        Some(Commands::Trim(args)) => trimmer::main(&args),
        Some(Commands::Gui(_)) => {
            // use a custom index.html to set the height of body to the full height of the window
            LaunchBuilder::desktop()
                .with_cfg(
                    Config::new()
                        .with_background_color((30, 30, 46, 255))
                        .with_menu(None)
                        // title bars suck
                        .with_window(WindowBuilder::new().with_decorations(false))
                        .with_disable_context_menu(true)
                        .with_custom_head("<style> #main { height: 100vh; } </style>".to_string()),
                )
                .launch(App);
        }
        _ => {}
    }
}
