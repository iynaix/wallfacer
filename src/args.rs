use clap::Parser;

// ------------------------- WALLPAPER UI -------------------------
#[derive(Parser, Debug)]
#[command(name = "wallpaper-ui", about = "Set wallpaper")]
pub struct WallpaperUIArgs {
    #[arg(long, action, help = "print version information and exit")]
    pub version: bool,

    // positional arguments for file paths
    pub paths: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
#[command(name = "wallpaper-ui", about = "Set wallpaper")]
pub struct WallpaperPipelineArgs {
    #[arg(long, action, help = "print version information and exit")]
    pub version: bool,
}
