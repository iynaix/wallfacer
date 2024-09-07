use clap::{builder::PossibleValuesParser, CommandFactory, Parser};
use std::path::PathBuf;
use wallfacer::{
    cli::ShellCompletion, config::WallpaperConfig, filter_images, is_image,
    pipeline::WallpaperPipeline,
};

// ------------------------- ADD WALLPAPERS -------------------------
#[derive(Parser, Debug)]
#[command(
    name = "add-wallpapers",
    about = "Adds wallpapers, and performs the face detection"
)]
pub struct WallpapersAddArgs {
    #[arg(
        long,
        action,
        help = "print version information and exit",
        exclusive = true
    )]
    pub version: bool,

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
    // positional arguments for file paths
    pub paths: Option<Vec<PathBuf>>,

    #[arg(
        long,
        value_enum,
        help = "type of shell completion to generate",
        hide = true,
        exclusive = true
    )]
    pub generate: Option<ShellCompletion>,
}

#[tokio::main]
async fn main() {
    let args = WallpapersAddArgs::parse();
    let cfg = WallpaperConfig::new();

    if args.version {
        println!("wallpapers-add {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(shell_completion) = args.generate {
        wallfacer::cli::generate_completions(
            "wallpapers-add",
            &mut WallpapersAddArgs::command(),
            &shell_completion,
        );
        return;
    }

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
        pipeline.add_image(&img, args.force);
    }

    pipeline.upscale_images();
    pipeline.optimize_images();
    pipeline.detect_faces().await;
    pipeline.preview();
}
