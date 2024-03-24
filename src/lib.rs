use args::WallpaperUIArgs;
use clap::Parser;
use itertools::Itertools;
use serde::Deserialize;
use std::{
    io::BufRead,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use wallpapers::Face;

pub mod args;
pub mod cropper;
pub mod geometry;
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

pub fn filename(path: &Path) -> String {
    path.file_name()
        .expect("could not get filename")
        .to_str()
        .expect("could not convert filename to str")
        .to_string()
}

// extend PathBuf with utility methods
pub trait PathBufExt {
    fn with_directory(&self, dir: &Path) -> PathBuf;
}

impl PathBufExt for PathBuf {
    fn with_directory(&self, dir: &Path) -> PathBuf {
        dir.join(self.file_name().expect("could not get filename"))
    }
}

pub fn filter_images(dir: &Path) -> impl Iterator<Item = PathBuf> {
    dir.read_dir()
        .unwrap_or_else(|_| panic!("could not read {:?}", &dir))
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    match ext.to_str() {
                        Some("jpg" | "jpeg" | "png") => return Some(path),
                        _ => return None,
                    }
                }
            }

            None
        })
}

pub fn get_paths_from_args() -> Vec<PathBuf> {
    let args = WallpaperUIArgs::parse();
    let mut all_files = Vec::new();
    if let Some(paths) = args.paths {
        paths.iter().flat_map(std::fs::canonicalize).for_each(|p| {
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

        all_files.extend(filter_images(&wallpaper_dir()));
    }

    // order by reverse chronological order
    all_files.iter().sorted_by_key(|f| {
        f.metadata()
            .expect("could not get file metadata")
            .modified()
            .expect("could not get file mtime")
    });
    all_files.reverse();

    all_files
}

#[derive(Debug, Deserialize)]
struct FaceJson {
    pub xmin: u32,
    pub xmax: u32,
    pub ymin: u32,
    pub ymax: u32,
}

impl FaceJson {
    pub const fn to_face(&self) -> Face {
        Face {
            xmin: self.xmin,
            xmax: self.xmax,
            ymin: self.ymin,
            ymax: self.ymax,
        }
    }
}

/// reads anime-face-detector's stdout one line at a time, yielding (filename, faces) pairs
pub fn detect_faces_iter(paths: &[PathBuf]) -> impl Iterator<Item = (String, Vec<Face>)> + '_ {
    let mut child = Command::new("anime-face-detector")
        .args(paths)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn anime-face-detector");

    let reader = std::io::BufReader::new(child.stdout.take().expect("failed to get stdout"));

    std::iter::zip(paths, reader.lines().map_while(Result::ok)).map(|(path, line)| {
        let faces: Vec<FaceJson> =
            serde_json::from_str(&line).expect("could not deserialize faces");
        let faces: Vec<_> = faces
            .into_iter()
            .map(|f: FaceJson| FaceJson::to_face(&f))
            .collect();

        let fname = filename(path);

        (fname, faces)
    })
}
