#[macro_use]
extern crate lazy_static;

use clap::Parser;
use core::panic;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use wallpaper_ui::{
    aspect_ratio::AspectRatio,
    cli::WallpapersAddArgs,
    config::WallpaperConfig,
    cropper::Cropper,
    detect_faces_iter, filename, filter_images, run_wallpaper_ui,
    wallpapers::{WallInfo, WallpapersCsv},
    PathBufExt,
};

lazy_static! {
    static ref CONFIG: WallpaperConfig = WallpaperConfig::new();
}

fn upscale_images(to_upscale: &[(PathBuf, u32)], format: &Option<String>) {
    to_upscale.par_iter().for_each(|(src, scale_factor)| {
        // let mut dest = src.with_directory(&CONFIG.wallpapers_path);
        let mut dest = src.with_directory("/tmp");

        if let Some(ext) = &format {
            dest = dest.with_extension(ext);
        }

        println!("Upscaling {}...", &filename(src));

        Command::new("realcugan-ncnn-vulkan")
            .arg("-i")
            .arg(src)
            .arg("-s")
            .arg(scale_factor.to_string())
            .arg("-o")
            .arg(dest)
            // silence output
            .stderr(Stdio::null())
            .spawn()
            .expect("could not spawn realcugan-ncnn-vulkan")
            .wait()
            .expect("could not wait for realcugan-ncnn-vulkan");
    });
}

fn optimize_webp(infile: &PathBuf, outfile: &PathBuf) {
    Command::new("cwebp")
        .args(["-q", "100", "-m", "6", "-mt", "-af"])
        .arg(infile)
        .arg("-o")
        .arg(outfile)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn cwebp")
        .wait()
        .expect("could not wait for cwebp");
}

fn optimize_jpg(infile: &PathBuf, outfile: &Path) {
    Command::new("jpegoptim")
        .arg("--strip-all")
        .arg(infile)
        .arg("--dest")
        .arg(
            outfile
                .parent()
                .unwrap_or_else(|| panic!("could not get parent directory for {infile:?}")),
        )
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn jpegoptim")
        .wait()
        .expect("could not wait for jpegoptim");
}

fn optimize_png(infile: &PathBuf, outfile: &PathBuf) {
    Command::new("oxipng")
        .args(["--opt", "max"])
        .arg(infile)
        .arg("--out")
        .arg(outfile)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn oxipng")
        .wait()
        .expect("could not wait for oxipng");
}

fn optimize_images(paths: &[PathBuf], format: &Option<String>) {
    let wall_dir = &CONFIG.wallpapers_path;
    for img in paths {
        println!("Optimizing {}...", filename(img));

        let out_img = format
            .as_ref()
            .map_or_else(|| img.clone(), |format| img.with_extension(format))
            .with_directory(wall_dir);

        if let Some(ext) = out_img.extension() {
            match ext.to_str().expect("could not convert extension to str") {
                "jpg" | "jpeg" => optimize_jpg(img, &out_img),
                "png" => optimize_png(img, &out_img),
                "webp" => optimize_webp(img, &out_img),
                _ => panic!("unsupported image format: {ext:?}"),
            }
        };
    }
}

// returns the faces that need to be previewed for selection
fn detect_faces(
    paths: &[PathBuf],
    wallpapers_csv: &mut WallpapersCsv,
    resolutions: &[AspectRatio],
) -> Vec<PathBuf> {
    if paths.is_empty() {
        return Vec::new();
    }

    for path in paths {
        println!("Detecting faces in {}...", filename(path));
    }

    let mut to_preview = Vec::new();

    for (path, faces) in detect_faces_iter(paths) {
        let fname = filename(path);
        let (width, height) = image::image_dimensions(path)
            .unwrap_or_else(|_| panic!("could not get image dimensions: {fname:?}"));
        let cropper = Cropper::new(&faces, width, height);

        // create WallInfo and save it
        let wall_info = WallInfo {
            filename: fname.clone(),
            width,
            height,
            faces,
            geometries: resolutions
                .iter()
                .map(|ratio| (ratio.clone(), cropper.crop(ratio)))
                .collect(),
            wallust: String::new(),
        };

        // preview both multiple faces and no faces
        if wall_info.faces.len() != 1 {
            to_preview.push(path.with_directory(&CONFIG.wallpapers_path));
        }

        wallpapers_csv.insert(fname.clone(), wall_info);
    }
    wallpapers_csv.save(resolutions);

    to_preview
}

/// waits for the images to be written to disk
fn wait_for_images(paths: &[PathBuf]) {
    while !paths.iter().all(|path| path.exists()) {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

fn main() {
    let args = WallpapersAddArgs::parse();
    let resolutions = CONFIG.sorted_resolutions();

    if args.version {
        println!("wallpapers-add {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let min_width: u32 = args.min_width.unwrap_or_else(|| CONFIG.min_width);
    let min_height: u32 = args.min_height.unwrap_or_else(|| CONFIG.min_height);

    let wall_dir = &CONFIG.wallpapers_path;
    if args.path == *wall_dir {
        eprintln!("Input directory cannot be the same as the wallpapers directory.");
        std::process::exit(1);
    }

    // create the csv if it doesn't exist
    let mut wallpapers_csv = WallpapersCsv::open().unwrap_or_default();

    // do a check for duplicates
    wallpapers_csv.find_duplicates();

    let mut to_upscale = Vec::new();
    let mut to_optimize = Vec::new();
    let mut to_detect = Vec::new();
    let mut to_preview = Vec::new();

    // add images from wallpapers dir that are not in the csv
    for img in filter_images(&wall_dir) {
        if wallpapers_csv.get(&filename(&img)).is_none() {
            to_optimize.push(img.clone());
            to_detect.push(img.clone());
        }
    }

    // get image dimensions of files within input_dir
    for img in filter_images(&args.path) {
        let (width, height) = image::image_dimensions(&img)
            .unwrap_or_else(|_| panic!("could not get image dimensions for {img:?}"));

        let out_path = args
            .format
            .as_ref()
            .map_or_else(|| img.clone(), |ext| img.with_extension(ext))
            .with_directory(wall_dir);

        if out_path.exists() {
            // check if corresponding WallInfo exists
            if let Some(info) = wallpapers_csv.get(&filename(&out_path)) {
                // re-preview if no / multiple faces detected and still using default crop
                if info.faces.len() != 1 && info.is_default_crops(&resolutions) {
                    to_preview.push(out_path);
                }
            // no WallInfo, redetect faces to write to csv
            } else {
                to_detect.push(out_path);
            }
        } else {
            if width * 4 < min_width || height * 4 < min_height {
                eprintln!(
                    "{:?} is too small to be upscaled to {min_width}x{min_height}",
                    &img
                );
                continue;
            }

            for scale_factor in 1..=4 {
                if width * scale_factor >= min_width && height * scale_factor >= min_height {
                    if scale_factor > 1 {
                        to_upscale.push((img, scale_factor));
                        to_optimize.push(out_path.with_directory("/tmp"));
                    } else {
                        to_optimize.push(img.clone());
                    }
                    to_detect.push(out_path);
                    break;
                }
            }
        }
    }

    upscale_images(&to_upscale, &args.format);
    wait_for_images(&to_optimize);

    optimize_images(&to_optimize, &args.format);
    wait_for_images(&to_detect);

    to_preview.extend(detect_faces(&to_detect, &mut wallpapers_csv, &resolutions));

    if !to_preview.is_empty() {
        run_wallpaper_ui(&to_preview);
    }
}
