#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    components::use_ui,
    state::{Wall, Wallpapers},
};

pub fn save_image(wall: &Wall, wallpapers: &mut Signal<Wallpapers>) {
    let mut ui = use_ui();

    wallpapers.with_mut(|wallpapers| {
        wall.current
            .save()
            .unwrap_or_else(|_| panic!("could not save {}", wall.current.path.display()));
        wallpapers.remove();
    });
    ui.with_mut(|ui| {
        ui.is_saving = true;
    });
}

#[component]
pub fn SaveButton(wall: Signal<Wall>, wallpapers: Signal<Wallpapers>) -> Element {
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
            class: "rounded-md px-5 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer {btn_color}",
            onclick: move |_| {
                save_image(&wall(), &mut wallpapers);
            },
            {btn_text}
        }
    }
}
