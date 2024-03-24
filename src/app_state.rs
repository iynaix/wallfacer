use wallpaper_ui::{cropper::AspectRatio, geometry::Geometry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiState {
    pub show_filelist: bool,
    pub show_faces: bool,
    pub manual_mode: bool,
    pub preview_geometry: Option<Geometry>,
    pub ratio: AspectRatio,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_filelist: Default::default(),
            show_faces: Default::default(),
            manual_mode: Default::default(),
            preview_geometry: Option::default(),
            ratio: AspectRatio(1440, 2560),
        }
    }
}

impl UiState {
    pub fn reset(&mut self) {
        self.show_filelist = false;
        self.show_faces = false;
        self.manual_mode = false;
        self.preview_geometry = None;
    }
}
