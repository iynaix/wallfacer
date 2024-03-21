#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::{AspectRatio, Geometry},
    wallpapers::Wallpapers,
};

// urls are relative to your Cargo.toml file
const _APP_URL: &str = manganis::mg!(file("./public/app.css"));
const _TAILWIND_URL: &str = manganis::mg!(file("./public/tailwind.css"));

pub mod align_group;
pub mod buttons;
pub mod candidates;
pub mod filelist;
pub mod preview;
pub mod switch;
pub mod toolbar;

use crate::{candidates::Candidates, filelist::FileList, preview::Previewer, toolbar::Toolbar};

fn main() {
    launch(App);
}

// define a component that renders a div with the text "Hello, world!"
fn App() -> Element {
    // let wall = "71124299_p0.png";
    let wall = "wallhaven-o567e7.jpg";

    let show_faces = use_signal(|| false);
    let show_filelist = use_signal(|| false);
    let wall_info = use_signal(|| Wallpapers::new()[wall].clone());
    let current_ratio = use_signal(|| AspectRatio(1440, 2560));
    let preview_geometry = use_signal::<Option<Geometry>>(|| None);

    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        link { rel: "stylesheet", href: "../public/app.css" },
        div {
            class: "dark bg-base p-4 gap-4 h-full flex overflow-hidden",

            if show_filelist() {
                FileList {
                    wall_info: wall_info,
                    show: show_filelist,
                    preview_geometry: preview_geometry,
                }
            } else {
                // main content
                div {
                    class: "flex flex-col gap-4 h-full",

                    Toolbar {
                        wall_info: wall_info,
                        current_ratio: current_ratio,
                        show_faces: show_faces,
                        show_filelist: show_filelist,
                    }

                    Previewer {
                        info: wall_info(),
                        ratio: current_ratio(),
                        show_faces: show_faces(),
                        preview_geometry: preview_geometry(),
                    }

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
