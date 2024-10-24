use clap::Args;
use itertools::Itertools;
use wallfacer::{
    aspect_ratio::AspectRatio,
    config::WallpaperConfig,
    cropper::Direction,
    geometry::Geometry,
    run_wallfacer,
    wallpapers::{WallInfo, WallpapersCsv},
};

pub fn add_geometry(info: &WallInfo, ratio: &AspectRatio, geom: Geometry) -> WallInfo {
    let mut new_geometries = info.geometries.clone();
    new_geometries.insert(ratio.clone(), geom);

    WallInfo {
        geometries: new_geometries,
        ..info.clone()
    }
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
        cfg.add_resolution(&name, new_res.clone());
        cfg.save().unwrap_or_else(|_| {
            eprintln!("Could not save config to {:?}!", cfg.csv_path);
            std::process::exit(1);
        });
    }

    let mut to_process: Vec<String> = Vec::new();
    let mut wallpapers_csv = WallpapersCsv::load(&cfg);

    let updated_infos = wallpapers_csv
        .iter()
        .map(|(fname, info)| {
            if info.geometries.contains_key(&new_res) {
                return info.clone();
            }

            let cropper = info.cropper();
            let default_crop = cropper.crop(&new_res);
            let updated_default_info = add_geometry(info, &new_res, default_crop.clone());

            match &closest_res {
                None => updated_default_info,
                Some(closest) => {
                    let closest_default_crop = cropper.crop(closest);

                    if info.direction(&default_crop) != info.direction(&closest_default_crop) {
                        return updated_default_info;
                    }

                    if info.get_geometry(closest) == closest_default_crop {
                        return updated_default_info;
                    }

                    // center new crop based on previous default crop
                    let new_geom = center_new_crop(&closest_default_crop, &default_crop, info);
                    to_process.push(fname.clone());
                    add_geometry(info, &new_res, new_geom)
                }
            }
        })
        .collect_vec();

    for updated_info in updated_infos {
        wallpapers_csv.insert(updated_info);
    }

    // update the csv
    wallpapers_csv.save();

    // open in wallfacer
    to_process.sort();
    let images = to_process
        .into_iter()
        .map(|fname| {
            println!("{fname}");

            cfg.wallpapers_dir
                .join(&fname)
                .to_str()
                .expect("could not convert path to str")
                .to_string()
        })
        .collect_vec();

    // process the images in wallfacer
    run_wallfacer(images);
}
