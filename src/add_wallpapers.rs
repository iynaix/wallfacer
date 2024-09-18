use std::path::PathBuf;

use clap::{builder::PossibleValuesParser, Args};
use wallfacer::{
    config::WallpaperConfig, filename, filter_images, is_image, pipeline::WallpaperPipeline,
    wallpapers::WallpapersCsv, PathBufExt,
};

#[derive(Args, Debug)]
pub struct AddWallpaperArgs {
    #[arg(
        long,
        action,
        value_name = "MIN_WIDTH",
        help = "minimum width for wallpapers to be resized, defaults to 1920 if not provided in config.ini"
    )]
    pub min_width: Option<u32>,

    #[arg(
        long,
        action,
        value_name = "MIN_HEIGHT",
        help = "minimum height for wallpapers to be resized, defaults to 1080 if not provided in config.ini"
    )]
    pub min_height: Option<u32>,

    #[arg(
        long,
        action,
        value_name = "FORMAT",
        value_parser = PossibleValuesParser::new(["jpg", "png", "webp"]),
        help = "optional format to convert the images to"
    )]
    pub format: Option<String>,

    #[arg(
        long,
        action,
        help = "reprocess the image even if it already exists in the csv"
    )]
    pub force: bool,

    // required positional argument for input directory
    /// directories or images to add
    pub paths: Vec<PathBuf>,
}

pub fn main(args: AddWallpaperArgs) {
    let cfg = WallpaperConfig::new();

    let wall_dir = &cfg.wallpapers_dir;
    let mut all_files = Vec::new();
    for p in args.paths.iter().flat_map(std::fs::canonicalize) {
        if let Some(p) = is_image(&p) {
            all_files.push(p);
        } else if p == *wall_dir {
            let wallpapers_csv = WallpapersCsv::open(&cfg).unwrap_or_default();
            let new_files = filter_images(&p)
                .filter(|p| wallpapers_csv.get(&filename(p)).is_none())
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
