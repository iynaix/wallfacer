#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    app_state::PreviewMode,
    components::{use_ui, use_wallpapers},
};

pub fn save_image() {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    let info = wallpapers().current;
    wallpapers.with_mut(|wallpapers| {
        wallpapers.insert_csv(&info);
        wallpapers.save_csv();

        wallpapers.remove();
    });
    ui.with_mut(|ui| {
        ui.preview_mode = PreviewMode::Candidate(None);
        ui.is_saving = true;
    });
}

#[component]
pub fn SaveButton() -> Element {
    let mut ui = use_ui();
    let clicked = ui().is_saving;

    use_future(move || async move {
        loop {
            if ui().is_saving {
                ui.with_mut(|ui| {
                    ui.is_saving = false;
                });
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let btn_color = if clicked {
        "bg-green-600"
    } else {
        "bg-indigo-600"
    };
    let btn_text = if clicked { "Saved" } else { "Save" };

    rsx! {
        a {
            class: "rounded-md px-5 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer",
            class: btn_color,
            onclick: move |_| {
                save_image();
            },
            {btn_text}
        }
    }
}
