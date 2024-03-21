#![allow(non_snake_case)]
use dioxus::prelude::*;
use itertools::Itertools;
use wallpaper_ui::{
    cropper::Geometry,
    wallpaper_dir,
    wallpapers::{WallInfo, Wallpapers},
};

#[derive(Clone, PartialEq, Props)]
pub struct WallpaperFileProps {
    filename: String,
    bytes: u64,
    onclick: EventHandler<MouseEvent>,
}

fn WallpaperFile(props: WallpaperFileProps) -> Element {
    let size_in_mb = format!("{:.2} MB", props.bytes as f64 / 1024.0 / 1024.0);

    rsx! {
        li {
            class: "flex justify-between gap-x-6 py-5",
            onclick: move |evt| {
                props.onclick.call(evt);
            },
            div { class: "flex min-w-0 gap-x-4",
                // TODO: thumbnail of wallpaper?
                // img {
                //     alt: "",
                //     src: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80",
                //     class: "h-12 w-12 flex-none rounded-full bg-gray-800"
                // }
                div { class: "min-w-0 flex-auto",
                    p { class: "text-sm font-semibold leading-6 text-white", {props.filename} }
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

#[derive(Clone, PartialEq, Props)]
pub struct FileListProps {
    class: Option<String>,
    wall_info: Signal<WallInfo>,
    show: Signal<bool>,
    preview_geometry: Signal<Option<Geometry>>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn FileList(mut props: FileListProps) -> Element {
    let mut search = use_signal(String::new);
    let normalized = search().to_lowercase();

    // handle esc to close file list
    // let handle_key_down_event = move |evt: KeyboardEvent| {
    //     if evt.key() == Key::Escape {
    //         props.show.set(false);
    //     }
    // };

    // get all filenames in the wallpaper directory
    let images = wallpaper_dir()
        .read_dir()
        .expect("could not read wallpaper directory")
        .flatten()
        // reverse chronological order
        .sorted_by(|a, b| {
            b.metadata()
                .expect("could not get file metadata")
                .modified()
                .expect("could not get file mtime")
                .cmp(
                    &a.metadata()
                        .expect("could not get file metadata")
                        .modified()
                        .expect("could not get file mtime"),
                )
        })
        .filter_map(|entry| {
            let path = entry.path();
            let Some(ext) = path.extension() else {
                return None;
            };

            if !(ext.eq_ignore_ascii_case("png")
                || ext.eq_ignore_ascii_case("jpg")
                || ext.eq_ignore_ascii_case("jpeg"))
            {
                return None;
            }

            // get file size
            let fname = entry
                .file_name()
                .to_str()
                .expect("could not get filename")
                .to_string();
            let size = entry.metadata().expect("could not get file metadata").len();

            // TODO: add number of faces?
            if search().is_empty() {
                return Some((fname, size));
            }

            if fname.to_lowercase().contains(&normalized) {
                Some((fname, size))
            } else {
                None
            }
        });

    rsx! {
        div {
            class: "flex flex-col flex-1 max-h-full overflow-y-auto {props.class.unwrap_or_default()}",
            // onkeydown: handle_key_down_event,

            // filter input
            div { class: "mt-2",
                div { class: "flex rounded-md bg-white/5 ring-1 ring-inset ring-white/10 focus-within:ring-2 focus-within:ring-inset focus-within:ring-indigo-500",
                    input {
                        r#type: "text",
                        placeholder: "Search",
                        name: "search_wallpapers",
                        class: "flex-1 border-0 bg-transparent py-1.5 pl-1 text-white focus:ring-0 sm:text-sm sm:leading-6",
                        id: "search_wallpapers",
                        oninput: move |evt| {
                            search.set(evt.value());
                        }
                    }
                }
            }

            ul { role: "list", class: "divide-y divide-gray-800",
                // HACK: render only the first 50 matches since there is no virtualized list
                for (fname, bytes) in images.take(50) {
                    WallpaperFile {
                        filename: fname.clone(),
                        bytes: bytes,
                        onclick: move |_| {
                            let wallpapers = Wallpapers::new();
                            let new_info = &wallpapers[fname.clone()];

                            props.wall_info.set(new_info.clone());
                            props.show.set(false);
                            props.preview_geometry.set(None);
                        },
                    }
                }
            }
        }
    }
}
