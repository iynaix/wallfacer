use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use itertools::Itertools;

use crate::filter_images;

use super::{
    Bbox, PathBufExt, config::Config, cropper::Cropper, run_wallfacer, wallpapers::WallInfo,
};

const WEBP_MAX_DIMENSION: u32 = 16383;

/// waits for the images to be written to disk
fn wait_for_image(path: &Path) {
    // wait for at most 5 minutes
    const TIMEOUT: Duration = Duration::from_secs(5 * 60);

    let start_time = Instant::now();
    while !path.exists() {
        if start_time.elapsed() >= TIMEOUT {
            eprintln!(
                "Timed out waiting for {} after {}",
                path.display(),
                TIMEOUT.as_secs()
            );
            std::process::exit(1);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn optimize_webp(
    infile: &PathBuf,
    outfile: &PathBuf,
) -> Result<std::process::ExitStatus, std::io::Error> {
    Command::new("cwebp")
        .args(["-q", "100", "-m", "6", "-mt", "-af"])
        .arg(infile)
        .arg("-o")
        .arg(outfile)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut c| c.wait())
}

pub fn optimize_jpg(
    infile: &PathBuf,
    outfile: &Path,
) -> Result<std::process::ExitStatus, std::io::Error> {
    Command::new("jpegoptim")
        .arg("--strip-all")
        .arg(infile)
        .arg("--dest")
        .arg(
            outfile.parent().unwrap_or_else(|| {
                panic!("could not get parent directory for {}", infile.display())
            }),
        )
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut c| c.wait())
}

pub fn optimize_png(
    infile: &PathBuf,
    outfile: &PathBuf,
) -> Result<std::process::ExitStatus, std::io::Error> {
    Command::new("oxipng")
        .args(["--opt", "max"])
        .arg(infile)
        .arg("--out")
        .arg(outfile)
        // silence output
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut c| c.wait())
}

#[derive(Default)]
pub struct WallpaperPipeline {
    config: Config,
    format: Option<String>,
    to_preview: Vec<PathBuf>,
}

impl WallpaperPipeline {
    pub fn new(cfg: &Config, format: Option<String>) -> Self {
        let wall_dir = &cfg.wallpapers_dir;

        // check that images from wallpapers dir all have metadata
        let orphan_wallpapers = filter_images(&wall_dir)
            .filter(|img| !WallInfo::has_metadata(img))
            .collect_vec();

        if !orphan_wallpapers.is_empty() {
            for img in orphan_wallpapers {
                eprintln!("orphan wallpaper: {}", img.display());
            }
            std::process::exit(1);
        }

        Self {
            format,
            config: cfg.clone(),
            to_preview: Vec::new(),
        }
    }

    pub fn add_image(&mut self, img: &PathBuf, force: bool, status_line: &str) {
        let (width, height) = image::image_dimensions(img)
            .unwrap_or_else(|_| panic!("could not get image dimensions for {}", img.display()));

        let out_path = self
            .format
            .as_ref()
            .map_or_else(|| img.clone(), |ext| img.with_extension(ext.as_str()))
            .with_directory(&self.config.wallpapers_dir);

        // detect -> upscale -> optimize -> save
        if out_path.exists() && !force {
            // check if corresponding WallInfo exists
            let info = WallInfo::new_from_file(&out_path);

            // image has been edited (different aspect ratio), re-process the image
            if info.width / width != info.height / height {
                self.detect(img, status_line);
                return;
            }

            // re-preview if no / multiple faces detected and still using default crop
            if info.faces.len() != 1 && info.is_default_crops(&self.config.sorted_resolutions()) {
                self.to_preview.push(out_path);
            }
        } else {
            self.detect(img, status_line);
        }
    }

    pub fn detect(&mut self, img: &PathBuf, status_line: &str) {
        print!("{status_line} Detecting faces...{}", " ".repeat(10));
        std::io::stdout().flush().expect("could not flush stdout");

        // get output of anime face detector
        let child = Command::new("anime-face-detector")
            .arg(img)
            .stdout(Stdio::piped())
            .output()
            .expect("failed to spawn anime-face-detector");

        let line = std::str::from_utf8(&child.stdout)
            .expect("could not convert output to str")
            .strip_suffix("\n")
            .unwrap_or_default()
            .to_string();

        let faces: Vec<Bbox> = serde_json::from_str(&line).expect("could not deserialize faces");
        let faces = faces.iter().map(|f: &Bbox| Bbox::to_face(f)).collect_vec();

        let (width, height) = image::image_dimensions(img)
            .unwrap_or_else(|_| panic!("could not get image dimensions: {}", img.display()));
        let cropper = Cropper::new(&faces, width, height);

        // create WallInfo and save it
        self.upscale(
            img,
            WallInfo {
                path: img.clone(),
                width,
                height,
                faces,
                geometries: self
                    .config
                    .sorted_resolutions()
                    .iter()
                    .map(|ratio| (ratio.clone(), cropper.crop(ratio)))
                    .collect(),
                ..Default::default()
            },
            status_line,
        );
    }

    pub fn upscale(&mut self, img: &PathBuf, info: WallInfo, status_line: &str) {
        let scale = info
            .get_scale(self.config.min_width, self.config.min_height)
            .unwrap_or_else(|| {
                eprintln!("{} is too small to be upscaled!", img.display());
                std::process::exit(1);
            });

        // edge case, webp too large
        assert!(
            !(info.width * scale > WEBP_MAX_DIMENSION || info.height * scale > WEBP_MAX_DIMENSION),
            "could not upscale {}, too large",
            img.display()
        );

        if scale == 1 {
            return self.optimize(img, &info, status_line);
        }

        print!("{status_line} Upscaling image...{}", " ".repeat(10));
        std::io::stdout().flush().expect("could not flush stdout");

        let mut dest = img.with_directory("/tmp");

        if let Some(ext) = &self.format {
            dest = dest.with_extension(ext);
        }

        Command::new("realcugan-ncnn-vulkan")
            .arg("-i")
            .arg(img)
            .arg("-s")
            .arg(scale.to_string())
            .arg("-o")
            .arg(&dest)
            // silence output
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut c| c.wait())
            .expect("could not run realcugan-ncnn-vulkan");

        self.optimize(&dest, &(info * scale), status_line);
    }

    pub fn optimize(&mut self, img: &PathBuf, info: &WallInfo, status_line: &str) {
        wait_for_image(img);

        print!("{status_line} Optimizing image...{}", " ".repeat(10));
        std::io::stdout().flush().expect("could not flush stdout");

        let out_img = self
            .format
            .as_ref()
            .map_or_else(|| img.clone(), |format| img.with_extension(format))
            .with_directory("/tmp");

        if let Some(ext) = out_img.extension().and_then(|ext| ext.to_str()) {
            (match ext {
                "jpg" | "jpeg" => optimize_jpg(img, &out_img),
                "png" => optimize_png(img, &out_img),
                "webp" => optimize_webp(img, &out_img),
                _ => panic!("unsupported image format: {ext:?}"),
            })
            .unwrap_or_else(|_| panic!("could not optimize {}", img.display()));
        }

        // save the image
        (WallInfo {
            path: out_img.clone(),
            ..info.clone()
        })
        .save()
        .unwrap_or_else(|_| panic!("could not save {}", out_img.display()));

        // copy final image with metadata to wallpapers dir
        std::fs::copy(
            &out_img,
            out_img.with_directory(&self.config.wallpapers_dir),
        )
        .unwrap_or_else(|_| panic!("could not copy {} to wallpapers dir", out_img.display()));

        // preview both multiple faces and no faces
        if info.faces.len() != 1 {
            let mut preview_img = img.with_directory(&self.config.wallpapers_dir);
            if let Some(ext) = &self.format {
                preview_img = preview_img.with_extension(ext);
            }
            self.to_preview.push(preview_img);
        }
    }

    pub fn preview(self) {
        if !self.to_preview.is_empty() {
            run_wallfacer(self.to_preview);
        }
    }
}
