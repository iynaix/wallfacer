use clap::{ArgGroup, CommandFactory, Parser};
use wallfacer::{
    aspect_ratio::AspectRatio,
    cli::ShellCompletion,
    config::WallpaperConfig,
    cropper::Direction,
    geometry::Geometry,
    run_wallfacer,
    wallpapers::{WallInfo, WallpapersCsv},
};

#[derive(Parser, Debug)]
#[command(name = "add-resolution", about = "Adds a new resolution for cropping",   group(
        ArgGroup::new("info")
            .args(&["version", "generate"])
    ))]
pub struct AddResolutionArgs {
    #[arg(
        long,
        action,
        help = "print version information and exit",
        exclusive = true,
        group = "info"
    )]
    pub version: bool,

    #[arg(
        long,
        value_enum,
        help = "type of shell completion to generate",
        hide = true,
        exclusive = true,
        group = "info"
    )]
    pub generate: Option<ShellCompletion>,

    #[arg(
        // help = "name of the new resolution",
        required_unless_present_any =["version", "generate"]
    )]
    pub name: Option<String>,

    #[arg(
        // help = "the new resolution, in the format <width>x<height>",
        required_unless_present_any = ["version", "generate"]
    )]
    pub resolution: Option<String>,
}

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

fn main() {
    let args = AddResolutionArgs::parse();

    if args.version {
        println!("add-resolution {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(shell_completion) = args.generate {
        wallfacer::cli::generate_completions(
            "add-resolution",
            &mut AddResolutionArgs::command(),
            &shell_completion,
        );
        return;
    }

    // the following checks shouldn't ever trigger as clap shouldn't allow it
    let name = args
        .name
        .unwrap_or_else(|| panic!("resolution name is required"));
    let resolution = args
        .resolution
        .unwrap_or_else(|| panic!("resolution is required"));

    let new_res = std::convert::TryInto::<AspectRatio>::try_into(resolution.as_str())
        .unwrap_or_else(|()| panic!("could not convert aspect ratio {} into string", resolution));

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

                    // center new crop based on previous default crop
                    let new_geom = center_new_crop(&closest_default_crop, &default_crop, info);
                    to_process.push(fname.clone());
                    add_geometry(info, &new_res, new_geom)
                }
            }
        })
        .collect();

    for updated_info in updated_infos {
        wallpapers_csv.insert(updated_info.filename.clone(), updated_info);
    }

    // update the csv
    wallpapers_csv.save(&cfg.sorted_resolutions());

    // open in wallfacer
    to_process.sort();
    let images: Vec<_> = to_process
        .into_iter()
        .map(|fname| {
            println!("{fname}");

            cfg.wallpapers_dir
                .join(&fname)
                .to_str()
                .expect("could not convert path to str")
                .to_string()
        })
        .collect();

    // process the images in wallfacer
    run_wallfacer(images);
}
