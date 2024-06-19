use std::path::PathBuf;

use clap::{builder::PossibleValuesParser, Parser};

// ------------------------- WALLPAPER UI -------------------------
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug)]
#[command(
    name = "wallpaper-ui",
    about = "Allows the selection of a cropping area for multiple monitor resolutions"
)]
pub struct WallpaperUIArgs {
    #[arg(long, action, help = "print version information and exit")]
    pub version: bool,

    #[arg(
        long,
        default_value = None,
        default_missing_value = "all",
        num_args = 0..=1,
        value_name = "RESOLUTIONS",
        help = "only show wallpapers that use the default crops; either \"all\" or resolution(s) in the format \"1920x1080,1920x1200\""
    )]
    pub unmodified: Option<String>,

    #[arg(
        long,
        default_value = None,
        default_missing_value = "all",
        num_args = 0..=1,
        value_name = "RESOLUTIONS",
        help = "only show wallpapers that don't use the default crops; either \"all\" or resolution(s) in the format \"1920x1080,1920x1200\""
    )]
    pub modified: Option<String>,

    #[arg(
        long,
        default_value = "all",
        default_missing_value = "all",
        value_parser = PossibleValuesParser::new([
            "zero",
            "none",
            "one",
            "single",
            "many",
            "multiple",
            "all",
        ]),
        help = "only show wallpapers that have a palette"
    )]
    pub faces: String,

    #[arg(long, help = "filters wallpapers by filename (case-insensitive)")]
    pub filter: Option<String>,

    // positional arguments for file paths
    pub paths: Option<Vec<PathBuf>>,
}

#[derive(Parser, Debug)]
#[command(
    name = "add-wallpapers",
    about = "Adds wallpapers, and performs the face detection"
)]
pub struct WallpapersAddArgs {
    #[arg(long, action, help = "print version information and exit")]
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

    // required positional argument for input directory
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(name = "add-resolution", about = "Adds a new resolution for cropping")]
pub struct AddResolutionArgs {
    #[arg(long, action, help = "print version information and exit")]
    pub version: bool,

    // required positional argument for input directory
    pub name: String,

    // required positional argument for input directory
    pub resolution: String,
}
