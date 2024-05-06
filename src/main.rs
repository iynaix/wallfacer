#![allow(non_snake_case)]
use clap::Parser;
use dioxus::desktop::Config;
use dioxus::prelude::*;
use wallpaper_ui::config::WallpaperConfig;

pub mod align_selector;
pub mod app_header;
pub mod app_state;
pub mod args;
pub mod button;
pub mod candidates;
pub mod drag_overlay;
pub mod dropdown;
pub mod filelist;
pub mod preview;
pub mod ratio_selector;
pub mod slider;
pub mod wallust;

use crate::{
    align_selector::AlignSelector,
    app_header::AppHeader,
    app_state::{UiState, Wallpapers},
    candidates::Candidates,
    filelist::FileList,
    preview::Previewer,
    ratio_selector::RatioSelector,
    wallust::Wallust,
};

fn main() {
    let args = args::WallpaperUIArgs::parse();
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

// define a component that renders a div with the text "Hello, world!"
fn App() -> Element {
    let config = WallpaperConfig::new();
    let wallpapers = use_signal(Wallpapers::from_args);
    let ui = use_signal(UiState::default);
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
            AppHeader { wallpapers, ui, resolutions: config.sorted_resolutions() }

            div {
                class: "flex p-4 gap-4",

                if (ui)().show_filelist {
                    FileList { wallpapers, ui }
                } else if (ui)().show_palette {
                    Wallust { wallpapers }
                } else {
                    // main content
                    div {
                        class: "flex flex-col gap-4 h-full",

                        // Toolbar
                        div {
                            class:"flex flex-row justify-between",
                            RatioSelector { wallpapers, ui, resolutions: config.resolutions },

                            div{
                                class: "flex justify-end",
                                AlignSelector { wallpapers, ui },
                            }
                        }

                        Previewer { wallpapers, ui }

                        Candidates { wallpapers, ui }
                    }
                }
            }
        }
    }
}
