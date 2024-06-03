use serde::Deserialize;
use std::{
    io::BufRead,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use wallpapers::Face;

pub mod aspect_ratio;
pub mod cli;
pub mod config;
pub mod cropper;
pub mod geometry;
pub mod wallpapers;

pub fn full_path(p: &str) -> PathBuf {
    p.strip_prefix("~/").map_or_else(
        || PathBuf::from(p),
        |p| {
            dirs::home_dir()
                .expect("could not get home directory")
                .join(p)
        },
    )
}

pub fn filename<P>(path: P) -> String
where
    P: AsRef<Path> + std::fmt::Debug,
{
    path.as_ref()
        .file_name()
        .unwrap_or_else(|| panic!("could not get filename: {:?}", path))
        .to_str()
        .unwrap_or_else(|| panic!("could not convert filename to str: {:?}", path))
        .to_string()
}

// extend PathBuf with utility methods
pub trait PathBufExt {
    fn with_directory<P>(&self, dir: P) -> PathBuf
    where
        P: AsRef<Path> + std::fmt::Debug;
}

impl PathBufExt for PathBuf {
    fn with_directory<P>(&self, path: P) -> PathBuf
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        path.as_ref().join(
            self.file_name()
                .unwrap_or_else(|| panic!("could not get filename for {path:?}")),
        )
    }
}

pub fn filter_images<P>(dir: P) -> impl Iterator<Item = PathBuf>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    dir.as_ref()
        .read_dir()
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
pub fn detect_faces_iter(paths: &[PathBuf]) -> impl Iterator<Item = (&PathBuf, Vec<Face>)> + '_ {
    let mut child = Command::new("anime-face-detector")
        .args(paths)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn anime-face-detector");

    let reader = std::io::BufReader::new(
        child
            .stdout
            .take()
            .expect("failed to get stdout of anime-face-detector"),
    );

    std::iter::zip(paths, reader.lines().map_while(Result::ok)).map(|(path, line)| {
        let faces: Vec<FaceJson> =
            serde_json::from_str(&line).expect("could not deserialize faces");
        let faces: Vec<_> = faces
            .into_iter()
            .map(|f: FaceJson| FaceJson::to_face(&f))
            .collect();

        (path, faces)
    })
}

pub fn run_wallpaper_ui<I, S>(args: I)
where
    I: IntoIterator<Item = S> + std::fmt::Debug + Clone,
    S: AsRef<std::ffi::OsStr>,
{
    if cfg!(debug_assertions) {
        Command::new("cargo")
            .args(["run", "--bin", "wallpaper-ui", "--"])
            .args(args)
            .spawn()
            .expect("could not spawn wallpaper-ui")
            .wait()
            .expect("could not wait for wallpaper-ui");
    } else {
        Command::new("wallpaper-ui")
            .args(args.clone())
            .spawn()
            .unwrap_or_else(|_| {
                // try running it via cargo instead
                Command::new("cargo")
                    .args(["run", "--release", "--bin", "wallpaper-ui", "--"])
                    .args(args)
                    .spawn()
                    .expect("could not spawn wallpaper-ui")
            })
            .wait()
            .expect("could not wait for wallpaper-ui");
    }
}
