use std::{io::Write, path::PathBuf};

use itertools::Itertools;
use wallfacer::{
    PathBufExt, PathBufNumericSort, cli::AddWallpaperArgs, config::Config, filter_images, is_image,
    pipeline::WallpaperPipeline, wallpapers::WallInfo,
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
                    std::fs::copy(&p, &target).unwrap_or_else(|_| {
                        panic!("could not copy image to /tmp: {}", p.display())
                    });

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
                .unwrap_or_else(|_| panic!("could not get image dimensions for {}", img.display()));
            width * 4 < cfg.min_width || height * 4 < cfg.min_height
        })
        .collect_vec();

    if !too_small.is_empty() {
        for img in too_small {
            eprintln!("{:?} is too small!", img.display());
        }
        std::process::exit(1);
    }

    let mut pipeline = WallpaperPipeline::new(&cfg, args.format.clone());
    let img_count = all_files.len();
    for (idx, img) in all_files.iter().enumerate() {
        let start_time = std::time::Instant::now();
        print!(
            "[{:0>width$}/{img_count}] Processing: {}\t",
            img.display(),
            idx + 1,
            width = img_count.to_string().len()
        );
        std::io::stdout().flush().expect("could not flush stdout");
        pipeline.add_image(img, args.force);
        println!("({:.3}s)", start_time.elapsed().as_secs_f64());
    }

    pipeline.preview();
}
