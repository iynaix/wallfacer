use serde::Deserialize;
use std::{
    io::BufRead,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use wallpapers::Face;

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
