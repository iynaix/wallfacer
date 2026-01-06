use std::path::PathBuf;

mod wall;
mod wallpapers;

// re-export
pub use wall::Wall;
pub use wallpapers::Wallpapers;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum UiMode {
    #[default]
    Editor,
    FileList,
    Adding(Vec<PathBuf>),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UiState {
    pub mode: UiMode,
    pub show_faces: bool,
    pub is_saving: bool,
    pub is_applying_wallpaper: bool,
    pub arrow_key_start: Option<std::time::Instant>,
}

impl UiState {
    pub fn toggle_filelist(&mut self) {
        self.mode = match self.mode {
            UiMode::FileList => UiMode::Editor,
            _ => UiMode::FileList,
        };
    }
}
