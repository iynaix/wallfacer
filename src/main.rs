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
            class: "m-16 mt-4",
            position: "relative",
            img {
                src: "{path}",
            }
            div {
                class: "absolute bg-black bg-opacity-60 {start_cls}",
                style: start_style,
            }
            div {
                class: "absolute bg-black bg-opacity-60 {end_cls}",
                style: end_style,
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct ButtonProps {
    class: Option<String>,
    text: String,
    onclick: EventHandler<MouseEvent>,
}

fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            "type": "button",
            class: "relative inline-flex items-center bg-mantle px-3 py-2 font-semibold text-text ring-1 ring-inset ring-gray-300 hover:bg-crust focus:z-10 {props.class.unwrap_or_default()}",
            onclick: move |evt| props.onclick.call(evt),
            {props.text},
        }
    }
}

#[component]
fn ResolutionSelector(current_resolution: Signal<(i32, i32)>) -> Element {
    let resolutions = [
        (1440, 2560),
        (2256, 1504),
        (3440, 1440),
        (1920, 1080),
        (1, 1),
    ];

    let buttons = resolutions.into_iter().enumerate().map(|(i, res)| {
        let cls = if i == 0 {
            "rounded-l-md"
        } else if i == resolutions.len() - 1 {
            "rounded-r-md"
        } else {
            "-ml-px"
        };

        rsx! {
            Button {
                class: "text-sm {cls}",
                text: format!("{}x{}", res.0, res.1),
                onclick: move |_| {
                    current_resolution.set(res);
                },
            }
        }
    });

    rsx! {
        span {
            class: "isolate inline-flex rounded-md shadow-sm",
            for button in buttons {
                {button}
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

    let current_resolution = use_signal(|| (1440, 2560));
    let geom = wall_info
        .get_geometry(current_resolution().0, current_resolution().1)
        .expect("could not get geometry");

    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        body {
            class: "dark bg-base",
            height: "100%",
            width: "100%",
            position: "absolute",
            flex: 1,

            div {
                class:"p-4",
                ResolutionSelector {
                    current_resolution: current_resolution,
                },
            }

            CropPreview {
                wall_info: wall_info.clone(),
                geometry: geom.try_into().expect("could not convert geometry"),
            }
        }
    }
}
