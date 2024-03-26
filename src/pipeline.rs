use clap::Parser;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use wallpaper_ui::{
    args::WallpaperPipelineArgs,
    cropper::{Cropper, FRAMEWORK_RATIO, HD_RATIO, SQUARE_RATIO, ULTRAWIDE_RATIO, VERTICAL_RATIO},
    detect_faces_iter, filename, filter_images, full_path, wallpaper_dir,
    wallpapers::{WallInfo, WallpapersCsv},
    PathBufExt,
};

const TARGET_WIDTH: u32 = 3440; // ultrawide width
const TARGET_HEIGHT: u32 = 1504; // framework height

fn upscale_images(to_upscale: &[(PathBuf, u32)]) {
    to_upscale.par_iter().for_each(|(src, scale_factor)| {
        let fname = filename(src);
        let mut dest = wallpaper_dir().join(&fname);
        // always output to png to avoid lossy compression
        dest.set_extension("png");

        println!("Upscaling {}...", &fname);

        Command::new("realcugan-ncnn-vulkan")
            .arg("-i")
            .arg(src)
            .arg("-s")
            .arg(scale_factor.to_string())
            .arg("-o")
            .arg(dest)
            // silence output
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("could not spawn realcugan-ncnn-vulkan")
            .wait()
            .expect("could not wait for realcugan-ncnn-vulkan");
    });
}

fn optimize_images(paths: &[PathBuf]) -> Vec<PathBuf> {
    let (pngs, jpgs): (Vec<_>, Vec<_>) = paths.iter().partition(|path| {
        path.extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("png"))
    });

    // jpegoptim for jpegs
    for jpg in &jpgs {
        println!("Optimizing {}...", filename(jpg));
    }

    Command::new("jpegoptim")
        .arg("--strip-all")
        .args(jpgs)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn jpegoptim")
        .wait()
        .expect("could not wait for jpegoptim");

    // oxipng for pngs
    for png in &pngs {
        println!("Optimizing {}...", filename(png));
    }

    Command::new("oxipng")
        .args(["--opt", "max"])
        .args(pngs)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn oxipng")
        .wait()
        .expect("could not wait for oxipng");

    paths.to_vec()
}

// returns the faces that need to be previewed for selection
fn detect_faces(paths: &[PathBuf], wallpapers_csv: &mut WallpapersCsv) -> Vec<PathBuf> {
    if paths.is_empty() {
        return Vec::new();
    }

    for path in paths {
        println!("Detecting faces in {}...", filename(path));
    }

    let mut to_preview = Vec::new();

    for (fname, faces) in detect_faces_iter(paths) {
        let cropper = Cropper::new(&fname, &faces);

        // create WallInfo and save it
        let vertical_crop = cropper.crop(&VERTICAL_RATIO);
        let wall_info = WallInfo {
            filename: fname.clone(),
            faces,
            r1440x2560: vertical_crop.clone(),
            r2256x1504: cropper.crop(&FRAMEWORK_RATIO),
            r3440x1440: cropper.crop(&ULTRAWIDE_RATIO),
            r1920x1080: cropper.crop(&HD_RATIO),
            r1x1: cropper.crop(&SQUARE_RATIO),
            wallust: String::new(),
        };

        if wall_info.faces.len() > 1 {
            to_preview.push(wallpaper_dir().join(&fname));
        }

        wallpapers_csv.insert(fname.clone(), wall_info);
    }
    wallpapers_csv.save();

    to_preview
}

fn get_output_path(img: &Path) -> Option<PathBuf> {
    let wall_dir = wallpaper_dir();
    for ext in &["png", "jpg", "jpeg"] {
        let output_path = img.with_extension(ext).with_directory(&wall_dir);
        if output_path.exists() {
            return Some(output_path);
        }
    }
    None
}

fn main() {
    let args = WallpaperPipelineArgs::parse();

    if args.version {
        println!("wallpaper-pipeline {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let input_dir = full_path("~/Pictures/wallpapers_in");
    let wall_dir = wallpaper_dir();
    let mut wallpapers_csv = WallpapersCsv::new();

    let mut to_copy = Vec::new();
    let mut to_upscale = Vec::new();
    let mut to_optimize = Vec::new();
    let mut to_detect = Vec::new();
    let mut to_preview = Vec::new();

    // get image dimensions of files within input_dir
    for img in filter_images(&input_dir) {
        let (width, height) =
            image::image_dimensions(&img).expect("could not get image dimensions");

        match get_output_path(&img) {
            Some(out_path) => {
                // check if corresponding WallInfo exists
                match wallpapers_csv.get(&filename(&out_path)) {
                    // re-preview if multiple faces detected and still using default crop
                    Some(info) => {
                        if info.faces.len() > 1 && info.is_default_crops() {
                            to_preview.push(img.clone());
                        }
                    }
                    // no WallInfo, redetect faces to write to csv
                    None => {
                        to_detect.push(img.clone());
                    }
                }
            }
            // no output file found, perform normal processing
            None => {
                for scale_factor in 1..=4 {
                    if width * scale_factor >= TARGET_WIDTH
                        && height * scale_factor >= TARGET_HEIGHT
                    {
                        if scale_factor > 1 {
                            let out_path = img.with_extension("png").with_directory(&wall_dir);
                            to_optimize.push(out_path);
                            to_upscale.push((img, scale_factor));
                        } else {
                            to_copy.push(img.clone());
                            to_optimize.push(img.with_directory(&wall_dir));
                        }
                        break;
                    }
                }
            }
        }
    }

    // copy images that don't need to be upscaled
    for img in &to_copy {
        std::fs::copy(img, img.with_directory(&wall_dir)).expect("could not copy file");
    }

    upscale_images(&to_upscale);

    to_detect.extend(optimize_images(&to_optimize));

    to_preview.extend(detect_faces(&to_detect, &mut wallpapers_csv));

    if !to_preview.is_empty() {
        let to_preview: Vec<_> = to_preview
            .iter()
            .map(|img| get_output_path(img).expect("could not get output path to preview"))
            .collect();

        if cfg!(debug_assertions) {
            Command::new("cargo")
                .args(["run", "--bin", "wallpaper-ui", "--"])
                .args(to_preview)
                .spawn()
                .expect("could not spawn wallpaper-ui")
                .wait()
                .expect("could not wait for wallpaper-ui");
        } else {
            Command::new("wallpaper-ui")
                .args(to_preview)
                .spawn()
                .expect("could not spawn wallpaper-ui")
                .wait()
                .expect("could not wait for wallpaper-ui");
        }
    }
}
