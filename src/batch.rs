use wallpaper_ui::{
    cropper::{Cropper, FRAMEWORK_RATIO, HD_RATIO, SQUARE_RATIO, ULTRAWIDE_RATIO, VERTICAL_RATIO},
    wallpapers::WallpapersCsv,
};

fn main() {
    // let wallpapers_csv = WallpapersCsv::new();

    // let multiple_faces = wallpapers_csv
    //     .iter()
    //     .filter(|(_, info)| info.faces.len() > 1);

    for (fname, info) in &WallpapersCsv::new() {
        println!("Processing {}", fname);

        for ratio in &[
            HD_RATIO,
            ULTRAWIDE_RATIO,
            VERTICAL_RATIO,
            FRAMEWORK_RATIO,
            SQUARE_RATIO,
        ] {
            Cropper::new(fname, &info.faces).crop2(ratio);
        }

        // wallpapers_csv.insert(fname.clone(), info.clone());
    }

    // wallpapers_csv.save();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_crop() {
        use wallpaper_ui::{
            cropper::{
                Cropper, FRAMEWORK_RATIO, HD_RATIO, SQUARE_RATIO, ULTRAWIDE_RATIO, VERTICAL_RATIO,
            },
            wallpapers::WallpapersCsv,
        };

        let wallpapers_csv = WallpapersCsv::new();

        for (fname, info) in &wallpapers_csv {
            let cropper = Cropper::new(fname, &info.faces);

            for ratio in &[
                HD_RATIO,
                ULTRAWIDE_RATIO,
                VERTICAL_RATIO,
                FRAMEWORK_RATIO,
                SQUARE_RATIO,
            ] {
                assert_eq!(cropper.crop(ratio), cropper.crop2(ratio));
            }
        }
    }

    // #[ignore]
    // #[test]
    // fn test_crop_candidates() {
    //     use wallpaper_ui::{
    //         cropper::{
    //             Cropper, FRAMEWORK_RATIO, HD_RATIO, SQUARE_RATIO, ULTRAWIDE_RATIO, VERTICAL_RATIO,
    //         },
    //         wallpapers::WallpapersCsv,
    //     };

    //     let wallpapers_csv = WallpapersCsv::new();

    //     for (fname, info) in &wallpapers_csv {
    //         // if fname != "99207867_p0.png" {
    //         //     continue;
    //         // }

    //         let cropper = Cropper::new(fname, &info.faces);

    //         for ratio in &[
    //             HD_RATIO,
    //             ULTRAWIDE_RATIO,
    //             VERTICAL_RATIO,
    //             FRAMEWORK_RATIO,
    //             SQUARE_RATIO,
    //         ] {
    //             println!("{}: {:?}", fname, ratio);
    //             println!("{:?}", &info.faces);

    //             // assert_eq!(cropper.crop(ratio), cropper.crop2(ratio));
    //         }
    //     }
    // }
}
