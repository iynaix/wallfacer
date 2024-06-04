#[macro_use]
extern crate lazy_static;

use clap::Parser;
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

fn upscale_images(to_upscale: &[(PathBuf, u32)]) {
    to_upscale.par_iter().for_each(|(src, scale_factor)| {
        // always output to png to avoid lossy compression
        let dest = src
            .with_directory(&CONFIG.wallpapers_path)
            .with_extension("png");

        println!("Upscaling {}...", &filename(src));

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

fn optimize_images(paths: &[PathBuf]) {
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

fn get_output_path(img: &Path) -> Option<PathBuf> {
    for ext in &["png", "jpg", "jpeg"] {
        let output_path = img
            .with_extension(ext)
            .with_directory(&CONFIG.wallpapers_path);
        if output_path.exists() {
            return Some(output_path);
        }
    }
    None
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

    let mut to_copy = Vec::new();
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

        if let Some(out_path) = get_output_path(&img) {
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
        }
        // no output file found, perform normal processing
        else {
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
                        let out_path = img.with_extension("png").with_directory(wall_dir);
                        to_optimize.push(out_path.clone());
                        to_upscale.push((img, scale_factor));
                        to_detect.push(out_path);
                    } else {
                        to_copy.push(img.clone());
                        to_optimize.push(img.with_directory(wall_dir));
                        to_detect.push(img);
                    }
                    break;
                }
            }
        }
    }

    // copy images that don't need to be upscaled
    for img in &to_copy {
        std::fs::copy(img, img.with_directory(wall_dir))
            .unwrap_or_else(|_| panic!("could not copy {img:?}"));
    }

    upscale_images(&to_upscale);

    optimize_images(&to_optimize);

    to_preview.extend(detect_faces(&to_detect, &mut wallpapers_csv, &resolutions));

    if !to_preview.is_empty() {
        let to_preview: Vec<_> = to_preview
            .iter()
            .map(|img| {
                get_output_path(img)
                    .unwrap_or_else(|| panic!("could not get output path of {img:?} to preview"))
            })
            .collect();

        run_wallpaper_ui(&to_preview);
    }
}
