use clap::Parser;
use indexmap::IndexMap;
use itertools::Itertools;
use std::path::PathBuf;

use crate::{FacesFilter, WallfacerArgs};

use wallfacer::{
    aspect_ratio::AspectRatio, config::WallpaperConfig, filename, filter_images, is_image,
    wallpapers::WallInfo,
};

use super::Wall;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpapers {
    pub files: Vec<PathBuf>,
    pub index: usize,
    pub ratio: AspectRatio,
    pub resolutions: IndexMap<String, AspectRatio>,
}

impl Wallpapers {
    /// parse an optional comma separated list of resolutions
    fn resolution_arg(
        resolution_arg: Option<&str>,
        resolutions: &[AspectRatio],
    ) -> Vec<AspectRatio> {
        match resolution_arg {
            None => Vec::new(),
            Some("all") => resolutions.to_vec(),
            Some(res_arg) => res_arg
                .split(',')
                .map(|s| {
                    std::convert::TryInto::<AspectRatio>::try_into(s.trim())
                        .unwrap_or_else(|_| panic!("Invalid resolution {s} provided."))
                })
                .collect(),
        }
    }

    pub fn from_args(cfg: &WallpaperConfig) -> Self {
        let args = WallfacerArgs::parse();
        let wall_dir = &cfg.wallpapers_dir;
        let resolutions = cfg.resolutions.clone().into_values().collect_vec();

        let mut modified_filters = Self::resolution_arg(args.modified.as_deref(), &resolutions);
        if !modified_filters.is_empty() {
            modified_filters = resolutions
                .iter()
                .filter(|r| !modified_filters.contains(r))
                .cloned()
                .collect();
        }

        let unmodified_filters = Self::resolution_arg(args.unmodified.as_deref(), &resolutions);

        let mut all_files: Vec<PathBuf> = Vec::new();
        if let Some(paths) = args.paths {
            paths.iter().flat_map(std::fs::canonicalize).for_each(|p| {
                if p.is_file() {
                    if let Some(p) = is_image(&p) {
                        all_files.push(p);
                    }
                } else {
                    all_files.extend(filter_images(&p));
                }
            });
        }

        if all_files.is_empty() {
            // defaults to wallpaper directory
            if !wall_dir.exists() {
                eprintln!("Wallpaper directory does not exist: {:?}", wall_dir);
                std::process::exit(1);
            }

            all_files.extend(filter_images(&wall_dir));
        }

        // filter only wallpapers that still use the default crops if needed
        all_files.retain(|f| {
            let info = WallInfo::new_from_file(f);
            if args.filter.is_some()
                && !filename(f).to_lowercase().contains(
                    &args
                        .filter
                        .as_ref()
                        .expect("no --filter provided")
                        .to_lowercase(),
                )
            {
                return false;
            }

            // check if wallpaper uses default crop for a resolution / all resolutions
            if !modified_filters.is_empty() {
                return info.is_default_crops(&modified_filters);
            }

            if !unmodified_filters.is_empty() {
                return info.is_default_crops(&unmodified_filters);
            }

            match args.faces {
                FacesFilter::All => true,
                FacesFilter::Zero | FacesFilter::None => info.faces.is_empty(),
                FacesFilter::One | FacesFilter::Single => info.faces.len() == 1,
                FacesFilter::Many | FacesFilter::Multiple => info.faces.len() > 1,
            }
        });

        // order by reverse chronological order
        all_files.sort_by_key(|f| {
            f.metadata()
                .unwrap_or_else(|_| panic!("could not get file metadata: {:?}", f))
                .modified()
                .unwrap_or_else(|_| panic!("could not get file mtime: {:?}", f))
        });
        all_files.reverse();

        Self {
            index: Default::default(),
            files: all_files,
            ratio: resolutions[0].clone(),
            resolutions: cfg.resolutions.clone(),
        }
    }

    pub fn current(&self) -> Wall {
        let path = self.files[self.index].clone();
        let info = WallInfo::new_from_file(&path);
        Wall::new(&info, path, &self.resolutions)
    }

    pub fn prev_wall(&mut self) {
        // loop back to the last wallpaper
        self.index = if self.index == 0 {
            self.files.len() - 1
        } else {
            self.index - 1
        };
    }

    pub fn next_wall(&mut self) {
        // loop back to the first wallpaper
        self.index = if self.index == self.files.len() - 1 {
            0
        } else {
            self.index + 1
        };
    }

    /// removes the current wallpaper from the list
    pub fn remove(&mut self) {
        let current_file = self.files[self.index].clone();
        if self.index == self.files.len() - 1 {
            self.files.remove(self.index);
            self.index = 0;
        } else {
            self.files.remove(self.index);
        }

        self.files.retain(|f| f != &current_file);
    }

    pub fn set_from_filename(&mut self, fname: &str) {
        self.index = self
            .files
            .iter()
            .position(|f| filename(f) == fname)
            .unwrap_or_else(|| panic!("could not find wallpaper: {}", fname));
    }
}

#[cfg(test)]
impl Wallpapers {
    pub fn create_mock(len: usize, index: usize) -> Self {
        assert!(index < len, "index must be less than len");

        let mut files = Vec::new();
        for i in 0..len {
            files.push(PathBuf::from(format!("{}", i)));
        }

        Self {
            files,
            index,
            ratio: AspectRatio { w: 16, h: 9 },
            resolutions: IndexMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_wall() {
        static LEN: usize = 5;
        let mut walls = Wallpapers::create_mock(LEN, 0);
        walls.next_wall();
        assert_eq!(walls.index, 1);
        assert_eq!(walls.files.len(), LEN);

        // loop around last element
        walls.index = LEN - 1;
        walls.next_wall();

        assert_eq!(walls.index, 0);
        assert_eq!(walls.files.len(), LEN);
    }

    #[test]
    fn test_prev_wall() {
        static LEN: usize = 5;
        let mut walls = Wallpapers::create_mock(LEN, 1);
        walls.prev_wall();
        assert_eq!(walls.index, 0);
        assert_eq!(walls.files.len(), LEN);

        // loop around first element
        walls.index = 0;
        walls.prev_wall();
        assert_eq!(walls.index, LEN - 1);
        assert_eq!(walls.files.len(), LEN);
    }

    #[test]
    fn test_remove_start() {
        static LEN: usize = 5;
        // remove first index
        let mut walls = Wallpapers::create_mock(LEN, 0);
        walls.remove();
        assert_eq!(walls.index, 0, "remove index 0");
        assert_eq!(walls.files.len(), LEN - 1, "remove index 0");
    }

    #[test]
    fn test_remove_middle() {
        // remove middle index
        static LEN: usize = 5;
        let mut walls = Wallpapers::create_mock(LEN, 2);
        walls.remove();
        assert_eq!(walls.index, 2, "remove index 2");
        assert_eq!(walls.files.len(), LEN - 1, "remove index 2");
    }

    #[test]
    fn test_remove_last() {
        // remove last index
        static LEN: usize = 5;
        let mut walls = Wallpapers::create_mock(LEN, LEN - 1);
        walls.remove();
        assert_eq!(walls.index, 0, "remove index {}", LEN - 1);
        assert_eq!(walls.files.len(), LEN - 1, "remove index {}", LEN - 1);

        walls.prev_wall();
        assert_eq!(walls.index, LEN - 1 - 1, "prev after remove last");
        assert_eq!(walls.files.len(), LEN - 1, "prev after remove last");
    }

    #[test]
    fn remove_single() {
        let mut walls = Wallpapers::create_mock(1, 0);
        walls.remove();
        assert_eq!(walls.files.len(), 0);
    }
}
