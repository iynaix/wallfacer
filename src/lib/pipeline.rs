use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use itertools::Itertools;

use super::{
    config::WallpaperConfig,
    cropper::Cropper,
    filename, filter_images, run_wallfacer,
    wallpapers::{WallInfo, WallpapersCsv},
    FaceJson, PathBufExt,
};

/// waits for the images to be written to disk
fn wait_for_image(path: &Path) {
    while !path.exists() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// get scale factor for the image
fn get_scale(width: u32, height: u32, min_width: u32, min_height: u32) -> Option<u32> {
    (1..=4).find(|&scale| width * scale >= min_width && height * scale >= min_height)
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

#[derive(Default)]
pub struct WallpaperPipeline {
    config: WallpaperConfig,
    format: Option<String>,
    wallpapers_csv: WallpapersCsv,
    to_preview: Vec<PathBuf>,
}

impl WallpaperPipeline {
    pub fn new(cfg: &WallpaperConfig, format: Option<String>) -> Self {
        // create the csv if it doesn't exist
        let wallpapers_csv = WallpapersCsv::open(cfg).unwrap_or_default();

        // do a check for duplicates
        wallpapers_csv.find_duplicates();

        let wall_dir = &cfg.wallpapers_dir;

        // add images from wallpapers dir that are not in the csv
        let orphan_wallpapers = filter_images(&wall_dir)
            .filter(|img| wallpapers_csv.get(img).is_none())
            .collect_vec();

        if !orphan_wallpapers.is_empty() {
            for img in orphan_wallpapers {
                eprintln!("orphan wallpaper: {img:?}");
            }
            std::process::exit(1);
        }

        Self {
            format,
            config: cfg.clone(),
            wallpapers_csv,
            to_preview: Vec::new(),
        }
    }

    pub fn save_csv(&mut self) {
        self.wallpapers_csv.save(&self.config.sorted_resolutions());
    }

    pub fn add_image(&mut self, img: &PathBuf, force: bool) {
        let (width, height) = image::image_dimensions(img)
            .unwrap_or_else(|_| panic!("could not get image dimensions for {img:?}"));

        let out_path = self
            .format
            .as_ref()
            .map_or_else(|| img.clone(), |ext| img.with_extension(ext.as_str()))
            .with_directory(&self.config.wallpapers_dir);

        let scale = get_scale(width, height, self.config.min_width, self.config.min_height);
        if out_path.exists() && !force {
            // check if corresponding WallInfo exists
            if let Some(info) = self.wallpapers_csv.get(&out_path) {
                // image has been edited, re-process the image
                if info.width / width != info.height / height {
                    match scale {
                        None => {
                            eprintln!("{img:?} is too small to be upscaled!");
                            std::process::exit(1);
                        }
                        Some(1) => self.optimize(img),
                        Some(scale) => self.upscale(img, scale),
                    }
                    return;
                }

                // re-preview if no / multiple faces detected and still using default crop
                if info.faces.len() != 1 && info.is_default_crops(&self.config.sorted_resolutions())
                {
                    self.to_preview.push(out_path);
                    return;
                }
            // no WallInfo, redetect faces to write to csv
            } else {
                self.detect(&out_path);
                return;
            }
            return;
        }

        match scale {
            None => {
                eprintln!("{img:?} is too small to be upscaled!");
                std::process::exit(1);
            }
            Some(1) => self.optimize(img),
            Some(scale) => self.upscale(img, scale),
        }
    }

    pub fn upscale(&mut self, img: &PathBuf, scale_factor: u32) {
        // nothing to do here
        let mut dest = img.with_directory("/tmp");

        if let Some(ext) = &self.format {
            dest = dest.with_extension(ext);
        }

        Command::new("realcugan-ncnn-vulkan")
            .arg("-i")
            .arg(img)
            .arg("-s")
            .arg(scale_factor.to_string())
            .arg("-o")
            .arg(&dest)
            // silence output
            .stderr(Stdio::null())
            .spawn()
            .expect("could not spawn realcugan-ncnn-vulkan")
            .wait()
            .expect("could not wait for realcugan-ncnn-vulkan");

        self.optimize(&dest);
    }

    pub fn optimize(&mut self, img: &PathBuf) {
        wait_for_image(img);

        let out_img = self
            .format
            .as_ref()
            .map_or_else(|| img.clone(), |format| img.with_extension(format))
            .with_directory(self.config.wallpapers_dir.clone());

        if let Some(ext) = out_img.extension() {
            match ext.to_str().expect("could not convert extension to str") {
                "jpg" | "jpeg" => optimize_jpg(img, &out_img),
                "png" => optimize_png(img, &out_img),
                "webp" => optimize_webp(img, &out_img),
                _ => panic!("unsupported image format: {ext:?}"),
            }
        };

        self.detect(&out_img);
    }

    pub fn detect(&mut self, img: &PathBuf) {
        wait_for_image(img);

        let fname = filename(img);

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

        let faces: Vec<FaceJson> =
            serde_json::from_str(&line).expect("could not deserialize faces");
        let faces: Vec<_> = faces
            .into_iter()
            .map(|f: FaceJson| FaceJson::to_face(&f))
            .collect();

        let (width, height) = image::image_dimensions(img)
            .unwrap_or_else(|_| panic!("could not get image dimensions: {img:?}"));
        let cropper = Cropper::new(&faces, width, height);

        // create WallInfo and save it
        let resolutions = self.config.sorted_resolutions();
        let wall_info = WallInfo {
            filename: fname,
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
            self.to_preview
                .push(img.with_directory(&self.config.wallpapers_dir));
        }

        self.wallpapers_csv.insert(wall_info);
        self.wallpapers_csv.save(&resolutions);
    }

    pub fn preview(self) {
        if !self.to_preview.is_empty() {
            run_wallfacer(self.to_preview);
        }
    }
}
