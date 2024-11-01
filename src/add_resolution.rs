use std::path::PathBuf;

use clap::Args;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use wallfacer::{
    aspect_ratio::AspectRatio,
    config::{Config, ConfigResolution},
    cropper::Direction,
    filter_images,
    geometry::Geometry,
    run_wallfacer,
    wallpapers::WallInfo,
};

/// adds and saves the new crop geometry
pub fn add_geometry(info: &mut WallInfo, aspect: &AspectRatio, geom: &Geometry) {
    info.geometries.insert(aspect.clone(), geom.clone());
    info.save()
        .unwrap_or_else(|_| panic!("could not save {}", info.path.display()));
}

/// centers the new crop based on the old crop
fn center_new_crop(closest_crop: &Geometry, new_crop: &Geometry, info: &WallInfo) -> Geometry {
    let direction = info.direction(closest_crop);
    let new_start = match direction {
        Direction::X => {
            f64::from(closest_crop.x) + f64::from(closest_crop.w) / 2.0
                - f64::from(new_crop.w) / 2.0
        }
        Direction::Y => {
            f64::from(closest_crop.y) + f64::from(closest_crop.h) / 2.0
                - f64::from(new_crop.h) / 2.0
        }
    };

    info.cropper()
        .clamp(new_start, direction, new_crop.w, new_crop.h)
}

#[derive(Args, Debug)]
pub struct AddResolutionArgs {
    /// name of the new resolution
    pub name: String,

    /// the new resolution, in the format <width>x<height>
    pub resolution: String,
}

// needed for parity with add_wallpapers in a match {}
pub fn main(args: &AddResolutionArgs) {
    // the following checks shouldn't ever trigger as clap shouldn't allow it
    let new_res = std::convert::TryInto::<AspectRatio>::try_into(args.resolution.as_str())
        .unwrap_or_else(|_| panic!("invalid aspect ratio: {} into string", args.resolution));

    let mut cfg = Config::new().expect("failed to load config");
    // finds the closest resolution to an existing one
    let closest_res = cfg
        .resolutions
        .iter()
        .min_by_key(|res| {
            let diff = OrderedFloat((f64::from(&res.resolution) - f64::from(&new_res)).abs());
            // ignore if aspect ratio already exists in config
            if diff == 0.0 {
                f64::INFINITY.into()
            } else {
                diff
            }
        })
        .map(|res| res.resolution.clone());

    // save the updated config
    if !cfg.resolutions.iter().any(|res| res.resolution == new_res) {
        cfg.resolutions.push(ConfigResolution {
            name: args.name.clone(),
            description: Some(args.name.clone()),
            resolution: new_res.clone(),
        });
        cfg.save().unwrap_or_else(|_| {
            eprintln!("Unable to add resolution to existing config, please do so manually.");
            std::process::exit(1);
        });
    }

    let mut to_process: Vec<PathBuf> = Vec::new();

    let all_files = filter_images(&cfg.wallpapers_dir).sorted();
    for path in all_files {
        println!("Processing {}", path.display());
        let mut info = WallInfo::new_from_file(&path);

        let cropper = info.cropper();
        let new_default_crop = cropper.crop(&new_res);

        match &closest_res {
            None => add_geometry(&mut info, &new_res, &new_default_crop),
            Some(closest) => {
                let closest_default_crop = cropper.crop(closest);

                // different direction
                if info.direction(&new_default_crop) != info.direction(&closest_default_crop) {
                    add_geometry(&mut info, &new_res, &new_default_crop);
                    to_process.push(path);
                    continue;
                }

                // the previous closest crop was not changed, just use the default
                if info.get_geometry(closest) == closest_default_crop {
                    add_geometry(&mut info, &new_res, &new_default_crop);
                    continue;
                }

                // center new crop based on previous default crop
                let new_geom = center_new_crop(&closest_default_crop, &new_default_crop, &info);
                // geometry was altered, skip
                if info.geometries.get(&new_res) != Some(&new_geom) {
                    continue;
                }

                to_process.push(path);
                add_geometry(&mut info, &new_res, &new_geom);
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
