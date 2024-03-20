#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::{Direction, Geometry},
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
    let (dir, start_pct, end_pct) = wall_info.overlay_percents(&geometry);

    let start_cls = match dir {
        Direction::X => "top-0 left-0 h-full",
        Direction::Y => "top-0 left-0 w-full",
    };
    let start_style = match dir {
        Direction::X => format!("width: {start_pct}%"),
        Direction::Y => format!("height: {start_pct}%"),
    };

    let end_cls = match dir {
        Direction::X => "top-0 right-0 h-full",
        Direction::Y => "bottom-0 left-0 w-full",
    };
    let end_style = match dir {
        Direction::X => format!("width: {end_pct}%"),
        Direction::Y => format!("height: {end_pct}%"),
    };

    rsx! {
        div {
            class: "m-16",
            position: "relative",
            img {
                src: "{path}",
            }
            div {
                class: "absolute bg-black bg-opacity-50 {start_cls}",
                style: start_style,
            }
            div {
                class: "absolute bg-black bg-opacity-50 {end_cls}",
                style: end_style,
            }
        }
    }
}

// define a component that renders a div with the text "Hello, world!"
fn App() -> Element {
    let wall = "71124299_p0.png";
    // let wall = "107610529_p1.png";

    let wallpapers = Wallpapers::new();
    let wall_info = &wallpapers[wall];
    let geom = wall_info
        // .get_geometry(1440, 2560)
        .get_geometry(3440, 1440)
        // .get_geometry(1, 1)
        .expect("could not get geometry");

    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        body {
            class: "bg-base",
            height: "100%",
            width: "100%",
            position: "absolute",
            flex: 1,

            CropPreview {
                wall_info: wall_info.clone(),
                geometry: geom.try_into().expect("could not convert geometry"),
            }
        }
    }
}
