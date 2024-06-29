use clap::Parser;
use core::panic;
use std::path::PathBuf;

use wallpaper_ui::{
    cli::WallpapersAddArgs,
    config::WallpaperConfig,
    filename, filter_images,
    image_ops::{detect_faces, optimize_images, upscale_images},
    is_image, run_wallpaper_ui,
    wallpapers::WallpapersCsv,
    PathBufExt,
};

/// waits for the images to be written to disk
fn wait_for_images(paths: &[PathBuf]) {
    while !paths.iter().all(|path| path.exists()) {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

#[tokio::main]
async fn main() {
    let args = WallpapersAddArgs::parse();
    let cfg = WallpaperConfig::new();
    let resolutions = cfg.sorted_resolutions();

    if args.version {
        println!("wallpapers-add {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let min_width: u32 = args.min_width.unwrap_or(cfg.min_width);
    let min_height: u32 = args.min_height.unwrap_or(cfg.min_height);

    let wall_dir = &cfg.wallpapers_path;
    let mut all_files = Vec::new();
    if let Some(paths) = args.paths {
        paths.iter().flat_map(std::fs::canonicalize).for_each(|p| {
            if p.is_file() {
                if let Some(p) = is_image(&p) {
                    all_files.push(p);
                }
            } else {
                if p == *wall_dir {
                    eprintln!("Input directory cannot be the same as the wallpapers directory.");
                    std::process::exit(1);
                }
                all_files.extend(filter_images(&p));
            }
        });
    }
    if all_files.is_empty() {
        eprintln!("No files found in input paths.");
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
    for img in all_files {
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

            continue;
        }

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

    upscale_images(&to_upscale, &args.format);
    wait_for_images(&to_optimize);

    println!("\n");
    optimize_images(&to_optimize, &args.format, wall_dir);
    wait_for_images(&to_detect);

    println!("\n");
    to_preview.extend(detect_faces(&to_detect, &mut wallpapers_csv, &resolutions, wall_dir).await);

    if !to_preview.is_empty() {
        run_wallpaper_ui(&to_preview);
    }
}
