use std::io::Write;

use itertools::Itertools;
use wallfacer::{
    PathBufVecExt, cli::AddWallpaperArgs, config::Config, pipeline::WallpaperPipeline,
};

pub fn main(args: &AddWallpaperArgs) {
    let cfg = Config::new().expect("failed to load config");
    let mut all_files = args.inputs.filter_wallpapers();
    all_files.numeric_sort();

    // check that all the files meet the minimum size requirement
    let too_small = all_files
        .iter()
        .filter(|img| {
            let (width, height) = image::image_dimensions(img)
                .unwrap_or_else(|_| panic!("could not get image dimensions for {}", img.display()));
            width * 4 < cfg.min_width || height * 4 < cfg.min_height
        })
        .collect_vec();

    if !too_small.is_empty() {
        for img in too_small {
            eprintln!("{:?} is too small!", img.display());
        }
        std::process::exit(1);
    }

    let mut pipeline = WallpaperPipeline::new(&cfg, args.format.clone(), args.output.clone());
    let img_count = all_files.len();
    for (idx, img) in all_files.iter().enumerate() {
        let start_time = std::time::Instant::now();
        let status_line = format!(
            "\r[{:0>width$}/{img_count}] Processing: {img}\t",
            idx + 1,
            width = img_count.to_string().len(),
            img = img.display()
        );
        std::io::stdout().flush().expect("could not flush stdout");
        pipeline.add_image(img, args.force, &status_line);
        // need the tabs at the end to overwrite the previous line cleanly
        println!(
            "{status_line} ({:.3}s){}",
            start_time.elapsed().as_secs_f64(),
            " ".repeat(15),
        );
    }

    pipeline.preview();
}
