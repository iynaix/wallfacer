#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::Geometry,
    wallpapers::{WallInfo, Wallpapers},
};

// urls are relative to your Cargo.toml file
const _TAILWIND_URL: &str = manganis::mg!(file("./public/tailwind.css"));

fn main() {
    launch(App);
}

#[component]
fn CropPreview(wall_info: WallInfo, geometry: Geometry) -> Element {
    let path = wall_info.path();
    let path = path.to_str().expect("could not convert path to str");
    let (start_pct, end_pct) = wall_info.overlay_percents(&geometry);

    rsx! {
        div {
            position: "relative",
            img {
                src: "{path}",
            }
            div {
                class: "absolute top-0 left-0 h-full bg-black bg-opacity-50",
                width: "{start_pct}%",
            }
            div {
                class: "absolute top-0 right-0 h-full bg-black bg-opacity-50",
                width: "{end_pct}%",
            }
        }
    }
}

// define a component that renders a div with the text "Hello, world!"
fn App() -> Element {
    let wall = "71124299_p0.png";

    let wallpapers = Wallpapers::new();
    let wall_info = &wallpapers[wall];
    let geom = wall_info
        .get_geometry(1440, 2560)
        .expect("could not get geometry");

    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        body {
            class: "bg-base",
            height: "100vh",
            width: "100vw",

            CropPreview {
                wall_info: wall_info.clone(),
                geometry: geom.into(),
            }
        }
    }
}
