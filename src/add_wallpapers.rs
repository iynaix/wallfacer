use std::path::PathBuf;

use clap::{builder::PossibleValuesParser, Args};
use wallfacer::{
    config::WallpaperConfig, filter_images, is_image, pipeline::WallpaperPipeline,
    wallpapers::WallInfo, PathBufExt,
};

#[derive(Args, Debug)]
pub struct AddWallpaperArgs {
    #[arg(
        long,
        action,
        value_name = "FORMAT",
        value_parser = PossibleValuesParser::new(["jpg", "png", "webp"]),
        help = "Optional format to convert the images to"
    )]
    pub format: Option<String>,

    #[arg(long, action, help = "Reprocess the image even if it already exists")]
    pub force: bool,

    // required positional argument for input directory
    /// directories or images to add
    pub paths: Vec<PathBuf>,
}

pub fn wallpapers_from_paths(paths: &[PathBuf], cfg: &WallpaperConfig) -> Vec<PathBuf> {
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

pub fn main(args: AddWallpaperArgs) {
    let cfg = WallpaperConfig::new();
    let mut all_files = wallpapers_from_paths(&args.paths, &cfg);
    all_files.sort();

    let mut pipeline = WallpaperPipeline::new(&cfg, args.format);
    for img in all_files {
        println!("Processing: {img:?}");
        pipeline.add_image(&img, args.force);
    }

    pipeline.preview();
}
