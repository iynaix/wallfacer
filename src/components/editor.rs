#![allow(non_snake_case)]
use dioxus::prelude::*;
use std::path::PathBuf;

use crate::{
    app_state::{UiState, Wallpapers},
    components::{
        align_selector::AlignSelector, candidates::Candidates, filelist::FileList,
        preview::Previewer, ratio_selector::RatioSelector, wallust::Wallust,
    },
};

#[component]
pub fn Editor(
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
    wallpapers_path: PathBuf,
) -> Element {
    rsx! {
        div {
            class: "flex p-4 gap-4",

            if (ui)().show_filelist {
                FileList { wallpapers, ui }
            } else if (ui)().show_palette {
                Wallust { wallpapers }
            } else {
                // main content
                div {
                    class: "flex flex-col gap-4 w-full h-full",

                    // Toolbar
                    div {
                        class:"flex flex-row justify-between",
                        RatioSelector { wallpapers, ui },

                        div{
                            class: "flex justify-end",
                            AlignSelector { wallpapers, ui },
                        }
                    }

                    Previewer { wallpapers, ui, wallpapers_path }

                    Candidates { wallpapers, ui }
                }
            }
        }
    }
}
