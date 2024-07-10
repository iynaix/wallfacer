#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::filename;

use crate::{
    app_state::PreviewMode,
    components::{use_ui, use_wallpapers},
};

#[component]
fn WallpaperFile(filename: String, bytes: u64, onclick: EventHandler<MouseEvent>) -> Element {
    let size_in_mb = format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0);

    rsx! {
        li {
            class: "flex justify-between gap-x-6 py-5 cursor-pointer",
            onclick: move |evt| {
                onclick.call(evt);
            },
            div { class: "flex min-w-0 gap-x-4",
                // TODO: thumbnail of wallpaper?
                // img {
                //     alt: "",
                //     src: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80",
                //     class: "h-12 w-12 flex-none rounded-full bg-gray-800"
                // }
                div { class: "min-w-0 flex-auto",
                    p { class: "text-sm font-semibold leading-6 text-white",
                        {filename}
                    }
                    p { class: "mt-1 truncate text-xs leading-5 text-gray-400",
                        { size_in_mb }
                    }
                }
            }
            // div { class: "hidden shrink-0 sm:flex sm:flex-col sm:items-end",
            //     p { class: "text-sm leading-6 text-white", "Co-Founder / CEO" }
            //     p { class: "mt-1 text-xs leading-5 text-gray-400",
            //         "Last seen "
            //         time { datetime: "2023-01-23T13:23Z", "3h ago" }
            //     }
            // }
        }
    }
}

#[component]
pub fn FileList(class: Option<String>) -> Element {
    let mut wallpapers = use_wallpapers();
    let mut ui = use_ui();

    let mut search = use_signal(String::new);
    let normalized = search().to_lowercase();

    let wallpaper_files = wallpapers().files;
    let images = wallpaper_files.iter().filter_map(|path| {
        let fname = filename(path);
        let size = path
            .metadata()
            .unwrap_or_else(|_| panic!("could not get file size for {fname}"))
            .len();

        if search().is_empty() {
            // TODO: add number of faces?
            return Some((fname, size));
        }

        if fname.to_lowercase().contains(&normalized) {
            // TODO: add number of faces?
            Some((fname, size))
        } else {
            None
        }
    });

    rsx! {
        div {
            class: "flex flex-col flex-1 max-h-full gap-4 {class.unwrap_or_default()}",
            // onkeydown: handle_key_down_event,

            // filter input
            div { class: "mt-2",
                div { class: "flex rounded-md bg-white/5 ring-1 ring-inset ring-white/10 focus-within:ring-2 focus-within:ring-inset focus-within:ring-indigo-500",
                    input {
                        r#type: "text",
                        placeholder: " Search",
                        name: "search_wallpapers",
                        class: "flex-1 border-0 bg-transparent py-1.5 pl-1 text-white focus:ring-0 sm:text-sm sm:leading-6",
                        id: "search_wallpapers",
                        oninput: move |evt| {
                            evt.stop_propagation();
                            search.set(evt.value());
                        }
                    }
                }
            }

            ul {
                role: "list",
                class: "divide-y divide-gray-800 overflow-y-auto mx-2",
                // HACK: render only the first 50 matches since there is no virtualized list
                for (fname, bytes) in images.take(50) {
                    WallpaperFile {
                        filename: fname.clone(),
                        bytes,
                        onclick: move |_| {
                            wallpapers.with_mut(|wallpapers| {
                                wallpapers.set_from_filename(&fname);
                            });
                            ui.with_mut(|ui| {
                                ui.preview_mode = PreviewMode::Candidate(None);
                                ui.toggle_filelist();
                            });
                        },
                    }
                }
            }
        }
    }
}
