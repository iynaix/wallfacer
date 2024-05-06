use std::path::PathBuf;

use ini::Ini;
use itertools::Itertools;

use crate::{cropper::AspectRatio, full_path};

#[derive(Debug, Clone, PartialEq)]
pub struct WallpaperConfig {
    pub wallpapers_path: PathBuf,
    pub csv_path: PathBuf,
    pub resolutions: Vec<(String, AspectRatio)>,
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        let wallpapers_path = full_path("~/Pictures/Wallpapers");

        Self {
            wallpapers_path: wallpapers_path.clone(),
            csv_path: wallpapers_path.join("wallpapers.csv"),
            resolutions: vec![("HD".into(), AspectRatio(1920, 1080))],
        }
    }
}

impl WallpaperConfig {
    pub fn new() -> Self {
        let config_file = dirs::config_dir()
            .expect("could not get xdg config directory")
            .join("wallpaper-ui/config.ini");

        if let Ok(conf) = Ini::load_from_file(config_file) {
            let resolutions = conf.section(Some("resolutions")).map_or_else(
                || Self::default().resolutions,
                |res| {
                    res.iter()
                        .map(|(k, v)| {
                            (
                                k.to_string(),
                                std::convert::TryInto::<AspectRatio>::try_into(v).unwrap_or_else(
                                    |()| panic!("could not convert aspect ratio {v} into string"),
                                ),
                            )
                        })
                        .sorted_by_key(|(_, ratio)| ratio.clone())
                        .collect()
                },
            );

            Self {
                wallpapers_path: conf
                    .general_section()
                    .get("wallpapers_path")
                    .map_or_else(|| Self::default().wallpapers_path, full_path),
                csv_path: conf
                    .general_section()
                    .get("csv_path")
                    .map_or_else(|| Self::default().csv_path, full_path),
                resolutions,
            }
        } else {
            Self::default()
        }
    }

    pub fn sorted_resolutions(&self) -> Vec<AspectRatio> {
        self.resolutions.iter().map(|(_, v)| v.clone()).collect()
    }
}
