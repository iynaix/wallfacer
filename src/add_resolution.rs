use std::path::PathBuf;

use itertools::Itertools;
use wallfacer::{
    PathBufVecExt,
    aspect_ratio::AspectRatio,
    cli::AddResolutionArgs,
    config::Config,
    cropper::Direction,
    filename, filter_images,
    geometry::Geometry,
    run_wallfacer,
    wallpapers::{WallInfo, save_preserving_modified},
};

/// adds and saves the new crop geometry
pub fn add_geometry(info: &mut WallInfo, aspect: &AspectRatio, geom: &Geometry) {
    save_preserving_modified(&info.path, |wallpaper_ns, meta| {
        meta.set_struct_field(
            wallpaper_ns,
            "crop",
            &aspect.to_string(),
            xmpkit::XmpValue::String(geom.to_string()),
        )
        .expect("Unable to add new crop");

        Ok(meta.clone())
    })
    .unwrap_or_else(|_| eprintln!("Error adding crop for {}", &info.path.display()));
}

/// centers the new crop based on the old crop
fn center_new_crop(
    closest_crop: &Geometry,
    new_crop: &Geometry,
    info: &WallInfo,
) -> (Geometry, bool) {
    let direction = info.direction(closest_crop);
    let new_start = match direction {
        Direction::X => {
            f64::from(closest_crop.x) + f64::from(closest_crop.w) / 2.0
                - f64::from(new_crop.w) / 2.0
        }
        // use the old crop's start, since that should be pretty close for vertical
        Direction::Y => f64::from(closest_crop.y),
    };

    let geom = info
        .cropper()
        .clamp(new_start, direction, new_crop.w, new_crop.h);

    // don't need to preview if crop is clamped to an edge
    let mut should_preview = true;

    if direction == Direction::X && (geom.x == 0 || geom.x == info.width - geom.w) {
        should_preview = false
    }
    if direction == Direction::Y && (geom.y == 0 || geom.y == info.height - geom.h) {
        should_preview = false
    }

    (geom, should_preview)
}

pub fn main(config_path: Option<PathBuf>, args: &AddResolutionArgs) {
    // the following checks shouldn't ever trigger as clap shouldn't allow it
    let new_res = std::convert::TryInto::<AspectRatio>::try_into(args.resolution.as_str())
        .unwrap_or_else(|_| panic!("invalid aspect ratio: {} into string", args.resolution));

    let cfg = Config::new(config_path).expect("failed to load config");
    // finds the closest resolution to an existing one
    let closest_res = cfg
        .resolutions
        .iter()
        // ignore the new resolution if already added to make the script idempotent
        .filter(|res| res.resolution != new_res)
        .min_by(|res1, res2| {
            let diff1 = (f64::from(&res1.resolution) - f64::from(&new_res)).abs();
            let diff2 = (f64::from(&res2.resolution) - f64::from(&new_res)).abs();
            // ignore if aspect ratio already exists in config
            diff1
                .partial_cmp(&diff2)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|res| res.resolution.clone());

    let all_files = filter_images(&args.input)
        .filter(|path| {
            if let Some(ref resume_from) = args.resume_from {
                return filename(path) >= *resume_from;
            }

            true
        })
        .collect_vec();

    let mut to_process: Vec<_> = all_files
        .into_iter()
        .filter(|path| {
            println!("Processing {}", path.display());
            let mut info = WallInfo::new_from_file(&path);

            let cropper = info.cropper();
            let new_default_crop = cropper.crop(&new_res);

            match &closest_res {
                None => {
                    add_geometry(&mut info, &new_res, &new_default_crop);
                    return false;
                }
                Some(closest) => {
                    let closest_crop = info.get_geometry(closest);

                    // different direction
                    if info.direction(&new_default_crop) != info.direction(&closest_crop) {
                        add_geometry(&mut info, &new_res, &new_default_crop);
                        return true;
                    }

                    // the previous closest crop was not changed, just use the default
                    if closest_crop == cropper.crop(closest) {
                        add_geometry(&mut info, &new_res, &new_default_crop);
                        return false;
                    }

                    // center new crop based on previous default crop
                    let (new_geom, should_preview) =
                        center_new_crop(&closest_crop, &new_default_crop, &info);
                    add_geometry(&mut info, &new_res, &new_geom);
                    return should_preview;
                }
            }
        })
        .collect();

    // open in wallfacer
    to_process.numeric_sort();
    let images = to_process
        .into_iter()
        .map(|path| path.display().to_string())
        .collect_vec();

    // process the images in wallfacer
    run_wallfacer(images);
}
