use clap::{builder::PossibleValuesParser, value_parser, Args, Parser, Subcommand, ValueEnum};
use std::{env, path::PathBuf};

// ------------------------- ADD WALLPAPERS -------------------------
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
    pub paths: Option<Vec<PathBuf>>,
}

// ------------------------- ADD RESOLUTION -------------------------
#[derive(Args, Debug)]
pub struct AddResolutionArgs {
    /// name of the new resolution
    pub name: Option<String>,

    /// the new resolution, in the format <width>x<height>
    pub resolution: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        name = "add",
        about = "Adds wallpapers with upscaling and face detection"
    )]
    Add(AddWallpaperArgs),

    #[command(name = "resolution", about = "Adds a new resolution for cropping")]
    AddResolution(AddResolutionArgs),
}

#[derive(ValueEnum, Debug, Clone)]
pub enum FacesFilter {
    Zero,
    None,
    One,
    Single,
    Many,
    Multiple,
    All,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[command(
    name = "wallfacer",
    about = "A GUI for selecting wallpaper cropping regions for multiple monitor resolutions, based on anime face detection.",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct WallfacerArgs {
    #[arg(
        long,
        value_enum,
        help = "type of shell completion to generate",
        hide = true,
        exclusive = true
    )]
    pub generate: Option<ShellCompletion>,

    #[command(subcommand)]
    pub command: Option<Commands>,

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
        value_parser = value_parser!(FacesFilter),
        help = "only show wallpapers that have a palette"
    )]
    pub faces: FacesFilter,

    #[arg(long, help = "filters wallpapers by filename (case-insensitive)")]
    pub filter: Option<String>,

    /// directories or images to add
    pub paths: Option<Vec<PathBuf>>,
}

// for generating shell completions
#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum ShellCompletion {
    Bash,
    Zsh,
    Fish,
}
