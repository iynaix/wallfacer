use clap::Parser;

// ------------------------- WALLPAPER UI -------------------------
#[derive(Parser, Debug)]
#[command(name = "wallpaper-ui", about = "Set wallpaper")]
pub struct WallpaperUIArgs {
    // positional arguments for file paths
    pub paths: Option<Vec<String>>,
}
