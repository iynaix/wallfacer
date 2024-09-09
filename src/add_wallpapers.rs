use crate::cli::AddWallpaperArgs;
use wallfacer::{config::WallpaperConfig, filter_images, is_image, pipeline::WallpaperPipeline};

pub fn add_wallpaper(args: AddWallpaperArgs) {
    let cfg = WallpaperConfig::new();

    let wall_dir = &cfg.wallpapers_dir;
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

    // allow loading and cleaning of wallpapers.csv
    let mut pipeline = WallpaperPipeline::new(
        &cfg,
        args.min_width.unwrap_or(cfg.min_width),
        args.min_height.unwrap_or(cfg.min_height),
        args.format,
    );

    if all_files.is_empty() {
        pipeline.save_csv();

        eprintln!("No files found in input paths.");
        std::process::exit(1);
    }

    for img in all_files {
        println!("Processing: {img:?}");
        pipeline.add_image(&img, args.force);
    }

    pipeline.preview();
}
