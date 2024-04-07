use wallpaper_ui::wallpapers::WallpapersCsv;

fn main() {
    let wallpapers_csv = WallpapersCsv::load();

    wallpapers_csv.save();
}
