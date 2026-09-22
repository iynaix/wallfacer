use wallfacer::{
    aspect_ratio::AspectRatio, cli::RemoveResolutionArgs, filter_images,
    wallpapers::save_preserving_modified,
};

pub fn main(args: &RemoveResolutionArgs) {
    // the following checks shouldn't ever trigger as clap shouldn't allow it
    let res = std::convert::TryInto::<AspectRatio>::try_into(args.resolution.as_str())
        .unwrap_or_else(|_| panic!("invalid aspect ratio: {} into string", args.resolution));

    for path in filter_images(&args.input) {
        println!("Processing {}", path.display());

        save_preserving_modified(&path, |meta| {
            meta.clear_tag(&format!("Xmp.wallfacer.crop.{}", res));

            Ok(())
        })
        .unwrap_or_else(|_| eprintln!("Error saving file: {}", path.display()))
    }
}
