use std::path::PathBuf;

pub mod cropper;
pub mod wallpapers;

pub fn full_path(p: &str) -> PathBuf {
    match p.strip_prefix("~/") {
        Some(p) => dirs::home_dir()
            .expect("could not get home directory")
            .join(p),
        None => PathBuf::from(p),
    }
}

pub fn wallpaper_dir() -> PathBuf {
    full_path("~/Pictures/Wallpapers")
}
