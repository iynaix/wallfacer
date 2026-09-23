use wallfacer::{cli::RemoveResolutionArgs, filter_images, wallpapers::save_preserving_modified};

pub fn main(args: &RemoveResolutionArgs) {
    for path in filter_images(&args.input) {
        println!("Processing {}", path.display());

        save_preserving_modified(&path, |wallfacer_ns, meta| {
            meta.delete_struct_field(wallfacer_ns, "crop", &args.resolution)
                .expect("unable to delete resolution");

            Ok(meta.clone())
        })
        .unwrap_or_else(|_| eprintln!("Error saving file: {}", path.display()))
    }
}
