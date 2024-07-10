#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_free_icons::{icons::md_device_icons::MdWallpaper, Icon};

use crate::components::{use_ui, use_wallpapers};

pub fn apply_wallpaper(wallpaper_cmd: &str, wall_path: &str) {
    let mut ui = use_ui();

    let wall_cmd = if wallpaper_cmd.contains("$1") {
        wallpaper_cmd.replace("$1", wall_path)
    } else {
        format!("{} {}", wallpaper_cmd, &wall_path)
    };

    std::process::Command::new("sh")
        .arg("-c")
        .arg(wall_cmd)
        .spawn()
        .expect("failed to set wallpaper");

    ui.with_mut(|ui| {
        ui.is_applying_wallpaper = true;
    });
}

#[component]
pub fn WallpaperButton(wallpaper_cmd: String) -> Element {
    let mut ui = use_ui();
    let wall_path = use_wallpapers()().full_path();
    let clicked = ui().is_applying_wallpaper;

    use_future(move || async move {
        loop {
            if ui().is_applying_wallpaper {
                ui.with_mut(|ui| {
                    ui.is_applying_wallpaper = false;
                });
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let btn_color = if clicked {
        "bg-green-600"
    } else {
        "bg-surface1 hover:bg-crust"
    };

    rsx! {
        a {
            class: "rounded-md px-3 py-2 text-sm font-semibold text-white shadow-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 cursor-pointer",
            class: btn_color,
            onclick: move |_| {
                apply_wallpaper(&wallpaper_cmd, &wall_path);
            },
            Icon { fill: "white", icon: MdWallpaper }
        }
    }
}
