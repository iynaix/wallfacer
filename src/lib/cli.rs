use clap::{Args, Parser, Subcommand, ValueEnum, builder::PossibleValuesParser};
use std::path::PathBuf;

// for generating shell completions
#[derive(Subcommand, ValueEnum, Debug, Clone)]
pub enum ShellCompletion {
    Bash,
    Zsh,
    Fish,
}

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

#[derive(Args, Debug)]
pub struct AddResolutionArgs {
    /// name of the new resolution
    pub name: String,

    /// the new resolution, in the format <width>x<height>
    pub resolution: String,
}

#[derive(Parser)]
#[command(
    name = "trimmer",
    about = "Automatic trimming of images",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct TrimmerArgs {
    #[arg(long, action, help = "Perform a trial run with no changes made")]
    pub dry_run: bool,

    #[arg(
        long,
        action,
        default_value = "false",
        help = "Trim the image horizontally"
    )]
    pub horizontal: bool,

    #[arg(long, action, default_value = "5.0", help = "Threshold for trimming")]
    pub threshold: f64,

    #[arg(help = "Directories or images to be trimmed", value_name = "PATHS")]
    pub paths: Option<Vec<PathBuf>>,
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

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        name = "add",
        about = "Adds wallpapers with upscaling and face detection"
    )]
    Add(AddWallpaperArgs),

    #[command(name = "resolution", about = "Adds a new resolution for cropping")]
    AddResolution(AddResolutionArgs),

    #[cfg(feature = "trimmer")]
    #[command(name = "trim", visible_alias = "crop", about = "Trims images")]
    Trim(TrimmerArgs),
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[command(
    name = "wallfacer",
    infer_subcommands = true,
    about = "A GUI for selecting wallpaper cropping regions for multiple monitor resolutions, based on anime face detection.",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct WallfacerArgs {
    #[arg(
        long,
        value_enum,
        help = "Type of shell completion to generate",
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
        help = "Only show wallpapers that use the default crops; either \"all\" or resolution(s) in the format \"1920x1080,1920x1200\""
    )]
    pub unmodified: Option<String>,

    #[arg(
        long,
        default_value = None,
        default_missing_value = "all",
        num_args = 0..=1,
        value_name = "RESOLUTIONS",
        help = "Only show wallpapers that don't use the default crops; either \"all\" or resolution(s) in the format \"1920x1080,1920x1200\""
    )]
    pub modified: Option<String>,

    #[arg(
        long,
        default_value = "all",
        default_missing_value = "all",
        value_parser = clap::value_parser!(FacesFilter),
        help = "Only show wallpapers that have a palette"
    )]
    pub faces: FacesFilter,

    #[arg(long, help = "Filters wallpapers by filename (case-insensitive)")]
    pub filter: Option<String>,

    #[arg(help = "Directories or images to be displayed", value_name = "PATHS")]
    pub paths: Option<Vec<PathBuf>>,
}
