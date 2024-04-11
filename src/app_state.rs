use clap::Parser;
use std::path::PathBuf;

use wallpaper_ui::{
    args::WallpaperUIArgs,
    cropper::AspectRatio,
    filename, filter_images, full_path,
    geometry::Geometry,
    wallpaper_dir,
    wallpapers::{WallInfo, WallpapersCsv},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UiState {
    pub show_filelist: bool,
    pub show_faces: bool,
    pub show_palette: bool,
    pub preview_mode: PreviewMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewMode {
    Manual,
    /// stores the last mouseover geometry
    Candidate(Option<Geometry>),
}

impl Default for PreviewMode {
    fn default() -> Self {
        Self::Candidate(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpapers {
    pub files: Vec<PathBuf>,
    // the original wallinfo before any modifications
    pub source: WallInfo,
    pub current: WallInfo,
    pub index: usize,
    pub ratio: AspectRatio,
}

impl Default for Wallpapers {
    fn default() -> Self {
        Self {
            files: Vec::default(),
            source: WallInfo::default(),
            current: WallInfo::default(),
            index: Default::default(),
            ratio: AspectRatio(1440, 2560),
        }
    }
}

impl Wallpapers {
    pub fn from_args() -> Self {
        let args = WallpaperUIArgs::parse();

        let mut all_files = Vec::new();
        if let Some(paths) = args.paths {
            paths
                .iter()
                .flat_map(|p| std::fs::canonicalize(full_path(p)))
                .for_each(|p| {
                    if p.is_file() {
                        all_files.push(p);
                    } else {
                        all_files.extend(filter_images(&p));
                    }
                });
        }

        if all_files.is_empty() {
            // defaults to wallpaper directory
            let wall_dir = wallpaper_dir();

            if !wall_dir.exists() {
                eprintln!("Wallpaper directory does not exist: {:?}", wall_dir);
                std::process::exit(1);
            }

            all_files.extend(filter_images(&wall_dir));
        }

        let wallpapers_csv = WallpapersCsv::load();

        // filter only wallpapers that still use the default crops if needed
        all_files.retain(|f| {
            let fname = filename(f);
            if let Some(info) = wallpapers_csv.get(&fname) {
                if args.filter.is_some()
                    && !fname.to_lowercase().contains(
                        &args
                            .filter
                            .as_ref()
                            .expect("no --filter provided")
                            .to_lowercase(),
                    )
                {
                    return false;
                }

                if args.only_unmodified && !info.is_default_crops() {
                    return false;
                }

                if args.only_single && info.faces.len() != 1 {
                    return false;
                }

                if args.only_none && !info.faces.is_empty() {
                    return false;
                }

                if args.only_multiple && info.faces.len() <= 1 {
                    return false;
                }
            }
            true
        });

        // order by reverse chronological order
        all_files.sort_by_key(|f| {
            f.metadata()
                .unwrap_or_else(|_| panic!("could not get file metadata: {:?}", f))
                .modified()
                .unwrap_or_else(|_| panic!("could not get file mtime: {:?}", f))
        });
        all_files.reverse();

        let fname = filename(
            all_files
                .first()
                .unwrap_or_else(|| panic!("no wallpapers found")),
        );
        let loaded = wallpapers_csv
            .get(&fname)
            .unwrap_or_else(|| panic!("could not get wallpaper info for {fname}"));

        Self {
            files: all_files,
            source: loaded.clone(),
            current: loaded.clone(),
            ..Default::default()
        }
    }

    pub fn prev_wall(&mut self) {
        // loop back to the last wallpaper
        self.index = if self.index == 0 {
            self.files.len() - 1
        } else {
            self.index - 1
        };

        let wallpapers_csv = WallpapersCsv::load();
        let fname = filename(&self.files[self.index]);
        let loaded = wallpapers_csv
            // bounds check is not necessary since the index is always valid
            .get(&fname)
            .unwrap_or_else(|| panic!("could not get wallpaper info for {fname}"));
        self.source = loaded.clone();
        self.current = loaded.clone();
    }

    pub fn next_wall(&mut self) {
        // loop back to the first wallpaper
        self.index = if self.index == self.files.len() - 1 {
            0
        } else {
            self.index + 1
        };

        let wallpapers_csv = WallpapersCsv::load();
        let fname = filename(&self.files[self.index]);
        let loaded = wallpapers_csv
            // bounds check is not necessary since the index is always valid
            .get(&filename(&self.files[self.index]))
            .unwrap_or_else(|| panic!("could not get wallpaper info for {fname}"));
        self.source = loaded.clone();
        self.current = loaded.clone();
    }

    /// removes the current wallpaper from the list
    pub fn remove(&mut self) {
        let current_index = self.index;
        self.next_wall();
        self.files.remove(current_index);
        // current_index is unchanged after removal
        self.index = current_index;
    }

    pub fn set_from_filename(&mut self, fname: &str) {
        let wallpapers_csv = WallpapersCsv::load();
        let loaded = wallpapers_csv
            .get(fname)
            .unwrap_or_else(|| panic!("could not get wallpaper info for {fname}"))
            .clone();
        self.source = loaded.clone();
        self.current = loaded;
        self.index = self
            .files
            .iter()
            .position(|f| filename(f) == fname)
            .unwrap_or_else(|| panic!("could not find wallpaper: {}", fname));
    }

    /// gets geometry for current aspect ratio
    pub fn get_geometry(&self) -> Geometry {
        self.current.get_geometry(&self.ratio)
    }

    /// sets the geometry for current aspect ratio
    pub fn set_geometry(&mut self, geom: &Geometry) {
        self.current.set_geometry(&self.ratio, geom);
    }

    /// returns crop candidates for current ratio and image
    pub fn crop_candidates(&self) -> Vec<Geometry> {
        self.current.cropper().crop_candidates(&self.ratio)
    }
}
