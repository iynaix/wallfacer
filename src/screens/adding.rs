#![allow(non_snake_case)]

use std::path::PathBuf;

use dioxus::prelude::*;

#[component]
pub fn Adding(images: Vec<PathBuf>) -> Element {
    println!("images: {:?}", images);

    return None;
}
