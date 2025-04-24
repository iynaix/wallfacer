#![allow(non_snake_case)]
use dioxus::prelude::*;

use wallfacer::config::Config;

use crate::{
    components::{app_header::AppHeader, save_button::save_image},
    screens::{
        adding::Adding,
        editor::{Editor, handle_arrow_keys_keyup, handle_editor_shortcuts},
        filelist::FileList,
        palette::Palette,
    },
    state::{UiMode, UiState, Wall, Wallpapers},
};

fn handle_shortcuts(
    evt: &Event<KeyboardData>,
    wall: &mut Signal<Wall>,
    wallpapers: &mut Signal<Wallpapers>,
    ui: &mut Signal<UiState>,
) {
    match evt.key() {
        Key::Character(shortcut) => {
            let shortcut = shortcut.as_str();

            match shortcut {
                "/" => {
                    ui.with_mut(UiState::toggle_filelist);
                }

                // ctrl+f
                "f" => {
                    if evt.modifiers().ctrl() {
                        ui.with_mut(UiState::toggle_filelist);
                    }
                }

                // ctrl+s
                "s" => {
                    if evt.modifiers().ctrl() && !wallpapers().files.is_empty() {
                        save_image(&wall(), wallpapers);
                    }
                }

                // palette
                // "p" => {
                //     if evt.modifiers().ctrl() && !wallpapers().files.is_empty() {
                //         ui.with_mut(app_state::UiState::toggle_palette);
                //     }
                // }
                _ => {
                    if ui().mode == UiMode::Editor {
                        handle_editor_shortcuts(evt, wall, wallpapers, ui);
                    }
                }
            }
        }
        _ => {
            if ui().mode == UiMode::Editor {
                handle_editor_shortcuts(evt, wall, wallpapers, ui);
            }
        }
    }
}

pub fn App() -> Element {
    let config =
        use_context_provider(|| Signal::new(Config::new().expect("failed to load config")));
    let wallpapers = use_signal(|| Wallpapers::from_args(&config()));

    if wallpapers().files.is_empty() {
        rsx! {
            document::Stylesheet {
                href: asset!("/public/tailwind.css")
            }

            main {
                class: "dark flex items-center h-full justify-center bg-base overflow-hidden",
                div {
                    h1 { class: "mt-4 text-4xl font-bold tracking-tight text-text text-center h-full",
                        "No more wallpapers to process! 🎉"
                    }
                }
            }
        }
    } else {
        rsx! {
            document::Stylesheet {
                href: asset!("/public/tailwind.css")
            }

            Main { config, wallpapers }
        }
    }
}

#[component]
fn Main(config: Signal<Config>, wallpapers: Signal<Wallpapers>) -> Element {
    let mut wall = use_signal(|| wallpapers().current());
    let mut ui = use_context_provider(|| {
        Signal::new(UiState {
            show_faces: config().show_faces,
            ..UiState::default()
        })
    });

    use_effect(move || {
        let mut new_wall = wallpapers().current();
        let prev_ratio = wall.peek().ratio.clone();

        let mut new_ratios = new_wall.ratios.iter().map(|r| r.resolution.clone());

        // prev ratio doesn't exist, just use first ratio returned by current()
        if !new_ratios.any(|r| r == prev_ratio) {
            wall.set(new_wall.clone());
            return;
        }

        // use the same ratio as before
        new_wall.ratio = prev_ratio;
        wall.set(new_wall);
    });

    rsx! {
        main {
            class: "dark flex flex-col h-full bg-base overflow-hidden",
            tabindex: 0,
            autofocus: true,
            onkeydown: move |evt| {
                handle_shortcuts(&evt, &mut wall, &mut wallpapers, &mut ui);
            },
            onkeyup: move |evt| {
                handle_arrow_keys_keyup(&evt.key(), &mut ui);
            },

            AppHeader { wall, wallpapers }

            div {
                class: "flex p-4 gap-4",

                if ui().mode == UiMode::FileList {
                    FileList { wallpapers }
                } else if ui().mode == UiMode::Palette {
                    Palette { wall }
                } else if ui().mode == UiMode::Editor {
                    Editor { wall }
                } else if let UiMode::Adding(images) = ui().mode {
                    Adding { images }
                }
            }
        }
    }
}
