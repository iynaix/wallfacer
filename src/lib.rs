use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    process::Command,
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

pub fn is_image<P>(path: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let p = path.as_ref();
    if p.is_file() {
        if let Some(ext) = p.extension() {
            match ext.to_str() {
                Some("jpg" | "jpeg" | "png" | "webp") => return Some(p.to_path_buf()),
                _ => return None,
            }
        }
    }

    None
}

pub fn filter_images<P>(dir: P) -> impl Iterator<Item = PathBuf>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    dir.as_ref()
        .read_dir()
        .unwrap_or_else(|_| panic!("could not read {:?}", &dir))
        .flatten()
        .filter_map(|entry| is_image(entry.path()))
}

#[derive(Debug, Deserialize)]
pub struct FaceJson {
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
