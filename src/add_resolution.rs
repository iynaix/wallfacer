use rexiv2::Metadata;
use std::path::PathBuf;

use clap::Args;
use itertools::Itertools;
use wallfacer::{
    aspect_ratio::AspectRatio, config::WallpaperConfig, cropper::Direction, filter_images,
    geometry::Geometry, run_wallfacer, wallpapers::WallInfo,
};

/// adds and saves the new crop geometry
pub fn add_geometry(info: &WallInfo, aspect: &AspectRatio, geom: &Geometry) {
    let meta = Metadata::new_from_path(&info.path).expect("could not init new metadata");

    // save the new crop metadata directly
    let crop_key = format!("Xmp.wallfacer.crop.{}", aspect);
    meta.set_tag_string(&crop_key, &geom.to_string())
        .unwrap_or_else(|_| panic!("could not set {crop_key}: {geom}"));
    meta.save_to_file(&info.path)
        .unwrap_or_else(|_| panic!("could not save metadata for {:?}", info.path));
}

/// centers the new crop based on the old crop
fn center_new_crop(old_crop: &Geometry, new_crop: &Geometry, info: &WallInfo) -> Geometry {
    let (crop_start, crop_length, direction) = match info.direction(old_crop) {
        Direction::X => (old_crop.x, old_crop.w, Direction::X),
        Direction::Y => (old_crop.y, old_crop.h, Direction::Y),
    };

    let closest_mid = f64::from(crop_start + crop_length) / 2.0;
    let default_start = closest_mid - f64::from(new_crop.w) / 2.0;
    info.cropper()
        .clamp(default_start, direction, new_crop.w, new_crop.h)
}

#[derive(Args, Debug)]
pub struct AddResolutionArgs {
    /// name of the new resolution
    pub name: Option<String>,

    /// the new resolution, in the format <width>x<height>
    pub resolution: Option<String>,
}

// needed for parity with add_wallpapers in a match {}
pub fn main(args: AddResolutionArgs) {
    // the following checks shouldn't ever trigger as clap shouldn't allow it
    let name = args
        .name
        .unwrap_or_else(|| panic!("resolution name is required"));
    let resolution = args
        .resolution
        .unwrap_or_else(|| panic!("resolution is required"));

    let new_res = std::convert::TryInto::<AspectRatio>::try_into(resolution.as_str())
        .unwrap_or_else(|_| panic!("invalid aspect ratio: {resolution} into string"));

    let mut cfg = WallpaperConfig::new();
    let closest_res = cfg.closest_resolution(&new_res);

    // save the updated config
    if !cfg.resolutions.iter().any(|(_, res)| res == &new_res) {
        cfg.add_resolution(&name, &new_res);
        cfg.save().unwrap_or_else(|_| {
            std::process::exit(1);
        });
    }

    let mut to_process: Vec<PathBuf> = Vec::new();

    for path in filter_images(&cfg.wallpapers_dir) {
        let info = WallInfo::new_from_file(&path);

        if info.geometries.contains_key(&new_res) {
            continue;
        }

        let cropper = info.cropper();
        let default_crop = cropper.crop(&new_res);

        match &closest_res {
            None => add_geometry(&info, &new_res, &default_crop),
            Some(closest) => {
                let closest_default_crop = cropper.crop(closest);

                if info.direction(&default_crop) != info.direction(&closest_default_crop) {
                    add_geometry(&info, &new_res, &default_crop);
                    continue;
                }

                if info.get_geometry(closest) == closest_default_crop {
                    add_geometry(&info, &new_res, &default_crop);
                    continue;
                }

                // center new crop based on previous default crop
                let new_geom = center_new_crop(&closest_default_crop, &default_crop, &info);
                to_process.push(path);
                add_geometry(&info, &new_res, &new_geom);
            }
        }
    }

    // open in wallfacer
    to_process.sort();
    let images = to_process
        .into_iter()
        .map(|path| path.display().to_string())
        .collect_vec();

    // process the images in wallfacer
    run_wallfacer(images);
}
