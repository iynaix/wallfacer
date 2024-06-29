use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{
    aspect_ratio::AspectRatio,
    cropper::Cropper,
    filename,
    wallpapers::{WallInfo, WallpapersCsv},
    FaceJson, PathBufExt,
};

pub fn upscale_images(to_upscale: &[(PathBuf, u32)], format: &Option<String>) {
    for (src, scale_factor) in to_upscale {
        let mut dest = src.with_directory("/tmp");

        if let Some(ext) = &format {
            dest = dest.with_extension(ext);
        }

        println!("Upscaling {}...", &filename(src));

        Command::new("realcugan-ncnn-vulkan")
            .arg("-i")
            .arg(src)
            .arg("-s")
            .arg(scale_factor.to_string())
            .arg("-o")
            .arg(dest)
            // silence output
            .stderr(Stdio::null())
            .spawn()
            .expect("could not spawn realcugan-ncnn-vulkan")
            .wait()
            .expect("could not wait for realcugan-ncnn-vulkan");
    }
}

pub fn optimize_webp(infile: &PathBuf, outfile: &PathBuf) {
    Command::new("cwebp")
        .args(["-q", "100", "-m", "6", "-mt", "-af"])
        .arg(infile)
        .arg("-o")
        .arg(outfile)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn cwebp")
        .wait()
        .expect("could not wait for cwebp");
}

pub fn optimize_jpg(infile: &PathBuf, outfile: &Path) {
    Command::new("jpegoptim")
        .arg("--strip-all")
        .arg(infile)
        .arg("--dest")
        .arg(
            outfile
                .parent()
                .unwrap_or_else(|| panic!("could not get parent directory for {infile:?}")),
        )
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn jpegoptim")
        .wait()
        .expect("could not wait for jpegoptim");
}

pub fn optimize_png(infile: &PathBuf, outfile: &PathBuf) {
    Command::new("oxipng")
        .args(["--opt", "max"])
        .arg(infile)
        .arg("--out")
        .arg(outfile)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not spawn oxipng")
        .wait()
        .expect("could not wait for oxipng");
}

/// optimize images in parallel
pub fn optimize_images(paths: &[PathBuf], format: &Option<String>, wall_dir: &PathBuf) {
    for img in paths {
        println!("Optimizing {}...", filename(img));

        let out_img = format
            .as_ref()
            .map_or_else(|| img.clone(), |format| img.with_extension(format))
            .with_directory(wall_dir);

        if let Some(ext) = out_img.extension() {
            match ext.to_str().expect("could not convert extension to str") {
                "jpg" | "jpeg" => optimize_jpg(img, &out_img),
                "png" => optimize_png(img, &out_img),
                "webp" => optimize_webp(img, &out_img),
                _ => panic!("unsupported image format: {ext:?}"),
            }
        };
    }
}

/// returns the faces that need to be previewed for selection
pub async fn detect_faces(
    paths: &[PathBuf],
    wallpapers_csv: &mut WallpapersCsv,
    resolutions: &[AspectRatio],
    wall_dir: &PathBuf,
) -> Vec<PathBuf> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    if paths.is_empty() {
        return Vec::new();
    }

    let mut to_preview = Vec::new();

    let mut child = Command::new("anime-face-detector")
        .args(paths)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn anime-face-detector");

    let reader = BufReader::new(
        child
            .stdout
            .take()
            .expect("failed to read stdout of anime-face-detector"),
    );
    let mut lines = reader.lines();
    let mut paths_iter = paths.iter();

    // read each line of anime-face-detector's output async
    while let (Some(path), Ok(Some(line))) = (paths_iter.next(), lines.next_line().await) {
        let fname = filename(path);
        println!("Detecting faces in {fname}...");

        let faces: Vec<FaceJson> =
            serde_json::from_str(&line).expect("could not deserialize faces");
        let faces: Vec<_> = faces
            .into_iter()
            .map(|f: FaceJson| FaceJson::to_face(&f))
            .collect();

        let (width, height) = image::image_dimensions(path)
            .unwrap_or_else(|_| panic!("could not get image dimensions: {fname:?}"));
        let cropper = Cropper::new(&faces, width, height);

        // create WallInfo and save it
        let wall_info = WallInfo {
            filename: fname.clone(),
            width,
            height,
            faces,
            geometries: resolutions
                .iter()
                .map(|ratio| (ratio.clone(), cropper.crop(ratio)))
                .collect(),
            wallust: String::new(),
        };

        // preview both multiple faces and no faces
        if wall_info.faces.len() != 1 {
            to_preview.push(path.with_directory(wall_dir));
        }

        wallpapers_csv.insert(fname, wall_info);
    }

    wallpapers_csv.save(resolutions);
    to_preview
}
