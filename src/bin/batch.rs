use wallpaper_ui::wallpapers::WallpapersCsv;

fn main() {
    let wallpapers_csv = WallpapersCsv::load();

    wallpapers_csv.save();

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
