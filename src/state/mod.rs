use std::path::PathBuf;

use wallfacer::geometry::Geometry;

mod wall;
mod wallpapers;

// re-export
pub use wall::Wall;
pub use wallpapers::Wallpapers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiMode {
    Editor,
    FileList,
    Palette,
    Adding(Vec<PathBuf>),
}

impl Default for UiMode {
    fn default() -> Self {
        Self::Editor
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UiState {
    pub mode: UiMode,
    pub show_faces: bool,
    pub is_saving: bool,
    pub is_applying_wallpaper: bool,
    pub arrow_key_start: Option<std::time::Instant>,
    pub mouseover_geom: Option<Geometry>,
}

impl UiState {
    pub fn toggle_filelist(&mut self) {
        self.mode = match self.mode {
            UiMode::FileList => UiMode::Editor,
            _ => UiMode::FileList,
        };
    }

    pub fn toggle_palette(&mut self) {
        self.mode = match self.mode {
            UiMode::Palette => UiMode::Editor,
            _ => UiMode::Palette,
        };
    }
}
