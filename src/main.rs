#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;

// urls are relative to your Cargo.toml file
const _TAILWIND_URL: &str = manganis::mg!(file("./public/tailwind.css"));

fn main() {
    launch(App);
}

// define a component that renders a div with the text "Hello, world!"
fn App() -> Element {
    rsx! {
        link { rel: "stylesheet", href: "../public/tailwind.css" },
        body {
            class: "bg-base",
            height: "100vh",
            width: "100vw",
            div {
                position: "relative",
                img {
                    src: "/home/iynaix/Pictures/Wallpapers/71124299_p0.png",
                }
                div {
                    class: "absolute top-0 left-0 h-full bg-black bg-opacity-50",
                    width: "20%",
                }
                div {
                    class: "absolute top-0 right-0 h-full bg-black bg-opacity-50",
                    width: "20%",
                }
            }
        }
    }
}
