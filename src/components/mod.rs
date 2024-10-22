pub mod align_buttons;
pub mod app_header;
pub mod button;
pub mod candidates;
pub mod drag_overlay;
pub mod dropdown;
pub mod preview;
pub mod ratio_buttons;
pub mod save_button;
pub mod slider;
pub mod wallpaper_button;

pub fn use_wallpapers() -> dioxus::signals::Signal<crate::state::Wallpapers> {
    dioxus::hooks::use_context()
}

pub fn use_ui() -> dioxus::signals::Signal<crate::state::UiState> {
    dioxus::hooks::use_context()
}
