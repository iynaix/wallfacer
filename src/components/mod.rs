pub mod align_selector;
pub mod app_header;
pub mod button;
pub mod candidates;
pub mod drag_overlay;
pub mod dropdown;
pub mod editor;
pub mod filelist;
pub mod palette;
pub mod preview;
pub mod ratio_selector;
pub mod save_button;
pub mod slider;
pub mod wallpaper_button;

pub fn use_wallpapers() -> dioxus::signals::Signal<crate::app_state::Wallpapers> {
    dioxus::hooks::use_context()
}

pub fn use_ui() -> dioxus::signals::Signal<crate::app_state::UiState> {
    dioxus::hooks::use_context()
}
