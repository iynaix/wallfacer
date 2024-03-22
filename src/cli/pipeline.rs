use rayon::prelude::*;
use regex::Regex;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use wallpaper_ui::{
    cropper::{Cropper, FRAMEWORK_RATIO, HD_RATIO, SQUARE_RATIO, ULTRAWIDE_RATIO, VERTICAL_RATIO},
    detect_faces_iter, filename, full_path,
    geometry::Geometry,
    wallpaper_dir,
    wallpapers::{WallInfo, WallpapersCsv},
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
            .expect("could not spawn process")
            .wait()
            .expect("could not wait for process");
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
        .expect("could not spawn process")
        .wait()
        .expect("could not wait for process");

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
        .expect("could not spawn process")
        .wait()
        .expect("could not wait for process");
}

// returns the faces that need to be previewed for selection
fn detect_faces(paths: &[PathBuf]) -> Vec<(String, Geometry)> {
    for path in paths {
        println!("Detecting faces in {}...", filename(path));
    }

    let mut wallpapers = WallpapersCsv::new();
    let mut to_preview = Vec::new();

    for (fname, faces) in detect_faces_iter(paths) {
        let mut cropper = Cropper::new(&fname, &faces);

        // create WallInfo and save it
        let vertical_crop = cropper.crop(&VERTICAL_RATIO).geometry();
        let wall_info = WallInfo {
            filename: fname.clone(),
            faces,
            r1440x2560: vertical_crop.clone(),
            r2256x1504: cropper.crop(&FRAMEWORK_RATIO).geometry(),
            r3440x1440: cropper.crop(&ULTRAWIDE_RATIO).geometry(),
            r1920x1080: cropper.crop(&HD_RATIO).geometry(),
            r1x1: cropper.crop(&SQUARE_RATIO).geometry(),
            wallust: String::new(),
        };

        if wall_info.faces.len() > 1 {
            to_preview.push((fname.clone(), vertical_crop));
        }

        wallpapers.insert(fname.clone(), wall_info);
    }

    wallpapers.save();

    // Wait for the command to finish
    // let output = child
    //     .wait_with_output()
    //     .unwrap_or_else(|_| panic!("anime-face-detector failed"));

    // Check if the command succeeded
    // if !output.status.success() {
    //     eprintln!(
    //         "anime-face-detector failed: {}",
    //         String::from_utf8_lossy(&output.stderr)
    //     );
    // }

    to_preview
}

fn create_previews(to_preview: &[(String, Geometry)]) {
    let preview_dir = full_path("~/projects/wallpaper-utils/in").join("preview");
    std::fs::create_dir_all(&preview_dir).expect("could not create directory");

    to_preview.par_iter().for_each(|(fname, geom)| {
        println!("Generating preview for {fname}...");

        let mut img = image::open(wallpaper_dir().join(fname)).expect("could not open image");
        let dest = preview_dir.join(fname.replace(".png", ".jpg"));
        img.crop(geom.x, geom.y, geom.w, geom.h)
            .save(dest)
            .expect("could not save preview image");
    });
}

fn main() {
    let input_dir = full_path("~/projects/wallpaper-utils/in");
    let mut to_copy = Vec::new();
    let mut to_upscale = Vec::new();
    let mut output_paths = Vec::new();
    let jpeg_re = Regex::new(r"(?i)\.(jpeg|jpg)$").expect("could not create jpeg regex");

    // get image dimensions of files within input_dir
    for entry in std::fs::read_dir(&input_dir).expect("could not read input_dir") {
        let path = entry.expect("could not get entry").path();

        if path.is_dir() {
            continue;
        }

        let fname = filename(&path);
        let (width, height) =
            image::image_dimensions(&path).expect("could not get image dimensions");

        for scale_factor in 1..=4 {
            if width * scale_factor >= TARGET_WIDTH && height * scale_factor >= TARGET_HEIGHT {
                if scale_factor > 1 {
                    let out_fname = jpeg_re.replace(&fname, ".png").to_string();
                    output_paths.push(wallpaper_dir().join(out_fname));
                    to_upscale.push((input_dir.join(fname), scale_factor));
                } else {
                    to_copy.push(fname.clone());
                    output_paths.push(wallpaper_dir().join(fname));
                }
                break;
            }
        }
    }

    // copy images that don't need to be upscaled
    for fname in &to_copy {
        let src = input_dir.join(fname);
        let dest = wallpaper_dir().join(fname);
        std::fs::copy(&src, &dest).expect("could not copy file");
    }

    upscale_images(&to_upscale);

    optimize_images(&output_paths);

    let to_preview = detect_faces(&output_paths);

    if !to_preview.is_empty() {
        println!("Creating previews...");
        create_previews(&to_preview);
    }
}
