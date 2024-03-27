use clap::Parser;

// ------------------------- WALLPAPER UI -------------------------
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug)]
#[command(name = "wallpaper-ui", about = "Set wallpaper")]
pub struct WallpaperUIArgs {
    #[arg(long, action, help = "print version information and exit")]
    pub version: bool,

    #[arg(
        long,
        default_value = "false",
        help = "only show wallpapers that still use the default crops"
    )]
    pub only_unmodified: bool,

    #[arg(
        long,
        default_value = "false",
        help = "only show wallpapers that have no faces detected"
    )]
    pub only_none: bool,

    #[arg(
        long,
        default_value = "false",
        help = "only show wallpapers that have a single face detected"
    )]
    pub only_single: bool,

    #[arg(
        long,
        default_value = "false",
        help = "only show wallpapers that have multiple faces detected"
    )]
    pub only_multiple: bool,

    // positional arguments for file paths
    pub paths: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
#[command(name = "wallpaper-ui", about = "Set wallpaper")]
pub struct WallpaperPipelineArgs {
    #[arg(long, action, help = "print version information and exit")]
    pub version: bool,
}
