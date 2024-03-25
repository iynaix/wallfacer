#![allow(non_snake_case)]
use dioxus::desktop::Config;
use dioxus::prelude::*;

// urls are relative to your Cargo.toml file
const _TAILWIND_URL: &str = manganis::mg!(file("./public/tailwind.css"));

pub mod align_selector;
pub mod app_header;
pub mod app_state;
pub mod args;
pub mod buttons;
pub mod candidates;
pub mod filelist;
pub mod preview;
pub mod ratio_selector;
pub mod switch;

use crate::{
    align_selector::AlignSelector,
    app_header::AppHeader,
    app_state::{UiState, Wallpapers},
    candidates::Candidates,
    filelist::FileList,
    preview::Previewer,
    ratio_selector::RatioSelector,
};

fn main() {
    // use a custom index.html to set the height of body to the full height of the window
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_custom_index(
                r#"<!DOCTYPE html>
<html>
    <head>
        <title>Dioxus app</title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
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
    let wallpapers = use_signal(Wallpapers::from_args);
    let ui_state = use_signal(UiState::default);

    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        div {
            class: "dark flex flex-col h-full bg-base overflow-hidden",
            AppHeader {
                wallpapers: wallpapers,
                ui: ui_state,
            }

            div {
                class: "flex p-4 gap-4",

                if (ui_state)().show_filelist {
                    FileList {
                        wallpapers: wallpapers,
                        ui: ui_state,
                    }
                } else {
                    // main content
                    div {
                        class: "flex flex-col gap-4 h-full",

                        // Toolbar
                        div {
                            class:"flex flex-row justify-between",

                            RatioSelector {
                                wallpapers: wallpapers,
                                ui: ui_state,
                            },

                            div{
                                class: "flex justify-end",

                                AlignSelector {
                                    // class: "ml-16 content-end",
                                    wallpapers: wallpapers,
                                    ui: ui_state,
                                },
                            }
                        }

                        Previewer {
                            wallpapers: wallpapers,
                            ui: ui_state,
                        }

                        Candidates {
                            wallpapers: wallpapers,
                            ui: ui_state,
                        }
                    }
                }
            }
        }
    }
}
