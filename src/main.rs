#![allow(non_snake_case)]
use dioxus::desktop::Config;
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::AspectRatio, filename, geometry::Geometry, get_paths_from_args,
    wallpapers::WallpapersCsv,
};

// urls are relative to your Cargo.toml file
const _TAILWIND_URL: &str = manganis::mg!(file("./public/tailwind.css"));

pub mod align_selector;
pub mod app_header;
pub mod args;
pub mod buttons;
pub mod candidates;
pub mod filelist;
pub mod preview;
pub mod resolution_selector;
pub mod switch;

use crate::{
    align_selector::AlignSelector, app_header::AppHeader, candidates::Candidates,
    filelist::FileList, preview::Previewer, resolution_selector::ResolutionSelector,
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
    let wallpaper_files = use_signal(get_paths_from_args);
    let wall_info = use_signal(|| {
        let fname = filename(
            &wallpaper_files
                .first()
                .expect("could not get first wallpaper"),
        );

        let wallpapers_csv = WallpapersCsv::new();
        wallpapers_csv
            .get(&fname)
            .expect("could not get wallpaper info")
            .clone()
    });
    let current_ratio = use_signal(|| AspectRatio(1440, 2560));

    let show_faces = use_signal(|| false);
    let show_filelist = use_signal(|| false);
    let manual_mode = use_signal(|| false);
    let preview_geometry = use_signal::<Option<Geometry>>(|| None);

    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        div {
            class: "dark flex flex-col h-full bg-base overflow-hidden",
            AppHeader {
                show_faces: show_faces,
                show_filelist: show_filelist,
                wall_info: wall_info,
                wallpaper_files: wallpaper_files,
            }

            div {
                class: "flex p-4 gap-4",

                if show_filelist() {
                    FileList {
                        paths: wallpaper_files(),
                        wall_info: wall_info,
                        show: show_filelist,
                        preview_geometry: preview_geometry,
                    }
                } else {
                    // main content
                    div {
                        class: "flex flex-col gap-4 h-full",

                        // Toolbar
                        div {
                            class:"flex flex-row justify-between",

                            ResolutionSelector {
                                current_ratio: current_ratio,
                                preview_geometry: preview_geometry,
                            },

                            div{
                                class: "flex justify-end",

                                AlignSelector {
                                    class: "ml-16 content-end",
                                    wall_info: wall_info,
                                    current_ratio: current_ratio(),
                                    manual_mode: manual_mode,
                                },
                            }
                        }

                        Previewer {
                            info: wall_info(),
                            ratio: current_ratio(),
                            show_faces: show_faces(),
                            manual_mode: manual_mode(),
                            preview_geometry: preview_geometry,
                        }

                        if !manual_mode() {
                            Candidates {
                                info: wall_info,
                                current_ratio: current_ratio(),
                                preview_geometry: preview_geometry,
                            }
                        }
                    }
                }
            }
        }
    }
}
