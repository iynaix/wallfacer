use std::path::PathBuf;

use ini::Ini;
use itertools::Itertools;

use crate::{aspect_ratio::AspectRatio, full_path};

#[derive(Debug, Clone, PartialEq)]
pub struct WallpaperConfig {
    pub wallpapers_path: PathBuf,
    pub csv_path: PathBuf,
    pub min_width: u32,
    pub min_height: u32,
    pub resolutions: Vec<(String, AspectRatio)>,
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        let wallpapers_path = full_path("~/Pictures/Wallpapers");
        let config_dir = dirs::config_dir()
            .expect("could not get xdg config directory")
            .join("wallpaper-ui");

        Self {
            wallpapers_path,
            csv_path: config_dir.join("wallpapers.csv"),
            min_width: 1920,
            min_height: 1080,
            resolutions: vec![("HD".into(), AspectRatio::new(1920, 1080))],
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
                                    |()| panic!("could not convert aspect ratio {v} from string"),
                                ),
                            )
                        })
                        .sorted_by_key(|(_, ratio)| ratio.clone())
                        .collect()
                },
            );

            let default_cfg = Self::default();

            Self {
                wallpapers_path: conf
                    .general_section()
                    .get("wallpapers_path")
                    .map_or_else(|| default_cfg.wallpapers_path, full_path),
                csv_path: conf
                    .general_section()
                    .get("csv_path")
                    .map_or_else(|| default_cfg.csv_path, full_path),
                min_width: conf.general_section().get("min_width").map_or_else(
                    || default_cfg.min_width,
                    |v| {
                        v.parse()
                            .unwrap_or_else(|_| panic!("invalid min_width {v} provided."))
                    },
                ),
                min_height: conf.general_section().get("min_width").map_or_else(
                    || default_cfg.min_width,
                    |v| {
                        v.parse()
                            .unwrap_or_else(|_| panic!("invalid min_width {v} provided."))
                    },
                ),
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
