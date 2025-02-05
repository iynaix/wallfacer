use std::path::PathBuf;

use itertools::Itertools;
use wallfacer::{
    cli::AddWallpaperArgs, config::Config, filter_images, is_image, pipeline::WallpaperPipeline,
    wallpapers::WallInfo, PathBufExt, PathBufNumericSort,
};

pub fn wallpapers_from_paths(paths: &[PathBuf], cfg: &Config) -> Vec<PathBuf> {
    let mut all_files = Vec::new();
    for p in paths.iter().flat_map(std::fs::canonicalize) {
        if let Some(p) = is_image(&p) {
            all_files.push(p);
        } else if p == cfg.wallpapers_dir {
            // add images from wallpapers_dir if they aren't processed yet
            let new_files = filter_images(&p)
                .filter(|p| !WallInfo::has_metadata(p))
                .map(|p| {
                    // copy to /tmp so pipeline can work on the copy instead of the original
                    let target = p.with_directory("/tmp");
                    std::fs::copy(&p, &target)
                        .unwrap_or_else(|_| panic!("could not copy image to /tmp: {p:?}"));

                    target
                });

            all_files.extend(new_files);
        } else {
            all_files.extend(filter_images(&p));
        }
    }

    all_files
}

pub fn main(args: &AddWallpaperArgs) {
    let cfg = Config::new().expect("failed to load config");
    let mut all_files = wallpapers_from_paths(&args.paths, &cfg);
    all_files.numeric_sort();

    // check that all the files meet the minimum size requirement
    let too_small = all_files
        .iter()
        .filter(|img| {
            let (width, height) = image::image_dimensions(img)
                .unwrap_or_else(|_| panic!("could not get image dimensions for {img:?}"));
            width * 4 < cfg.min_width || height * 4 < cfg.min_height
        })
        .collect_vec();

    if !too_small.is_empty() {
        for img in too_small {
            eprintln!("{img:?} is too small!");
        }
        std::process::exit(1);
    }

    let mut pipeline = WallpaperPipeline::new(&cfg, args.format.clone());
    for img in all_files {
        println!("Processing: {img:?}");
        pipeline.add_image(&img, args.force);
    }

    pipeline.preview();
}
