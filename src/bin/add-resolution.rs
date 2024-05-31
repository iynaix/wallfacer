use ordered_float::OrderedFloat;
use wallpaper_ui::{
    aspect_ratio::AspectRatio,
    config::WallpaperConfig,
    cropper::Direction,
    geometry::Geometry,
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

fn main() {
    // TODO: parse from command line arg
    let resolution_arg = "1920x2880";
    let new_res = std::convert::TryInto::<AspectRatio>::try_into(resolution_arg)
        .unwrap_or_else(|()| panic!("could not convert aspect ratio {resolution_arg} into string"));

    let mut config_ratios = WallpaperConfig::new().sorted_resolutions();
    let closest_res = config_ratios.iter().min_by_key(|res| {
        let diff = OrderedFloat((f64::from(*res) - f64::from(&new_res)).abs());
        // ignore if aspect ratio already exists in config
        if diff == 0.0 {
            f64::INFINITY.into()
        } else {
            diff
        }
    });

    let mut to_process: Vec<String> = Vec::new();
    let mut wallpapers_csv = WallpapersCsv::load();

    let updated_infos: Vec<WallInfo> = wallpapers_csv
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

                    let new_geom = match info.direction(&closest_default_crop) {
                        Direction::X => {
                            let closest_mid =
                                f64::from(closest_default_crop.x + closest_default_crop.w) / 2.0;
                            let default_start = closest_mid - f64::from(default_crop.w) / 2.0;
                            let geom = cropper.clamp(
                                default_start,
                                Direction::X,
                                default_crop.w,
                                default_crop.h,
                            );

                            if !(geom.x == 0 || geom.x == info.width - geom.w) {
                                to_process.push(fname.clone());
                            }

                            geom
                        }
                        Direction::Y => {
                            let closest_mid =
                                f64::from(closest_default_crop.y + closest_default_crop.h) / 2.0;
                            let default_start = closest_mid - f64::from(default_crop.h) / 2.0;
                            let geom = cropper.clamp(
                                default_start,
                                Direction::Y,
                                default_crop.w,
                                default_crop.h,
                            );

                            if !(geom.y == 0 || geom.y == info.height - geom.h) {
                                to_process.push(fname.clone());
                            }

                            geom
                        }
                    };
                    add_geometry(info, &new_res, new_geom)
                }
            }
        })
        .collect();

    for updated_info in updated_infos {
        wallpapers_csv.insert(updated_info.filename.clone(), updated_info);
    }
    // add the new resolution
    config_ratios.push(new_res.clone());
    config_ratios.sort();

    wallpapers_csv.save(&config_ratios);

    println!("To process: {:?}", to_process.len());
}
