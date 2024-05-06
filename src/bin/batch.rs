use wallpaper_ui::{config::WallpaperConfig, wallpapers::WallpapersCsv};

fn main() {
    let config = WallpaperConfig::new();
    let wallpapers_csv = WallpapersCsv::load();

    wallpapers_csv.save(&config.sorted_resolutions());

    // let argstr = [
    //     "wallust",
    //     "--backend",
    //     "wal",
    //     "--palette",
    //     "dark",
    //     "--fallback-generator",
    //     "complementary",
    // ];
}

// #[cfg(test)]
// mod tests {
//     use clap::Parser;

//     #[test]
//     fn test_wallust_to() {}
// }
