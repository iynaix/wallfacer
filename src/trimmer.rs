// trimming images

use std::path::{Path, PathBuf};

use clap::Parser;
use image::{GenericImageView, ImageBuffer, ImageReader, Rgb};
use rayon::prelude::*;
use wallfacer::{filename, filter_images, is_image};

#[allow(clippy::cast_lossless)]
fn mean(data: &[i32]) -> f64 {
    let sum = data.iter().sum::<i32>() as f64;
    let count = data.len();
    assert!(count > 0, "data must have at least one element");

    sum / count as f64
}

#[allow(clippy::cast_lossless)]
fn std_deviation(data: &[i32]) -> f64 {
    let count = data.len();
    assert!(count > 0, "data must have at least one element");

    let data_mean = mean(data);
    let variance = data
        .iter()
        .map(|value| {
            let diff = data_mean - (*value as f64);
            diff * diff
        })
        .sum::<f64>()
        / count as f64;

    variance.sqrt()
}

struct Trimmer {
    threshold: f64,
    horizontal: bool,
}

impl Trimmer {
    fn is_trimmable(&self, pixels: impl Iterator<Item = Rgb<u8>>) -> bool {
        let mut reds: Vec<i32> = Vec::new();
        let mut greens: Vec<i32> = Vec::new();
        let mut blues: Vec<i32> = Vec::new();

        for px in pixels {
            reds.push(px[0].into());
            greens.push(px[1].into());
            blues.push(px[2].into());
        }

        std_deviation(&reds) < self.threshold
            && std_deviation(&greens) < self.threshold
            && std_deviation(&blues) < self.threshold
    }

    #[allow(clippy::needless_pass_by_value)]
    fn find_trimmable<Outer, Inner, GetPixelFn>(
        &self,
        mut outer_range: Outer,
        inner_range: Inner,
        get_pixel: GetPixelFn,
    ) -> u32
    where
        Outer: Iterator<Item = u32>,
        Inner: Iterator<Item = u32> + Clone,
        GetPixelFn: Fn(u32, u32) -> Rgb<u8>,
    {
        let first = outer_range.next().expect("could not get first row / col");
        let second = outer_range.next().expect("could not get second row / col");

        // closure for checking row / colum
        let trimmable = |outer| {
            let pixels = inner_range.clone().map(|inner| get_pixel(outer, inner));
            !self.is_trimmable(pixels)
        };

        // try skipping the edgemost pixels to see if it produces a better crop
        if trimmable(first) && trimmable(second) {
            first
        } else {
            // continue searching from the second pixel onwards
            outer_range
                .find(|&outer| trimmable(outer))
                .expect("image is completely trimmed")
        }
    }

    fn trimmed_area(&self, img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> (u32, u32, u32, u32) {
        let width = img.width();
        let height = img.height();

        // let y_start = find_trimmable!(
        let y_start = self.find_trimmable(0..height, 0..width, |y, x| *img.get_pixel(x, y));
        // add 1 due to non-inclusive range
        // let y_end = find_trimmable!(
        let y_end = self.find_trimmable((y_start..height).rev(), 0..width, |y, x| {
            *img.get_pixel(x, y)
        }) + 1;

        // use y_start..y_end as those rows would already be cropped anyway
        let x_start = if self.horizontal {
            self.find_trimmable(0..width, y_start..y_end, |x, y| *img.get_pixel(x, y))
        } else {
            0
        };
        // add 1 due to non-inclusive range
        let x_end = if self.horizontal {
            self.find_trimmable((x_start..width).rev(), y_start..y_end, |x, y| {
                *img.get_pixel(x, y)
            }) + 1
        } else {
            width
        };

        // x, y, width, height
        (x_start, y_start, x_end - x_start, y_end - y_start)
    }

    fn trim(&self, wall: &PathBuf, output: &Path) {
        println!("Processing: {wall:?}");

        let img = ImageReader::open(wall)
            .expect("could not open image")
            .decode()
            .expect("could not decode image")
            .to_rgb8();

        let (x, y, width, height) = self.trimmed_area(&img);

        // let (img_width, img_height) = img.dimensions();
        // if img_width != width || img_height != height {
        //     println!("{wall:?} {img_width}x{img_height} -> {width}x{height}");
        // }

        // nothing to trim
        // if width == img.width() && height == img.height() {
        //     std::fs::copy(wall, out).expect("could not copy file");
        //     return;
        // }

        let cropped = img.view(x, y, width, height).to_image();
        cropped
            .save(&output.join(filename(wall)))
            .unwrap_or_else(|_| panic!("could not save trimmed image for {wall:?}"));
    }
}

#[derive(Parser)]
#[command(
    name = "trimmer",
    about = "Automatic trimming of images",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct TrimmerArgs {
    #[arg(
        long,
        action,
        default_value = "false",
        help = "Trim the image horizontally"
    )]
    pub horizontal: bool,

    #[arg(long, action, default_value = "5.0", help = "Threshold for trimming")]
    pub threshold: f64,

    #[arg(help = "Directory or image to be trimmed", value_name = "INPUT_DIR")]
    pub input: PathBuf,

    #[arg(help = "Directory to output trimmed images", value_name = "OUTPUT_DIR")]
    pub output: PathBuf,
}

pub fn main(args: &TrimmerArgs) {
    let mut all_files = Vec::new();
    std::fs::canonicalize(&args.input).map_or_else(
        |_| {
            eprintln!("Could not find input file /directory");
            std::process::exit(1);
        },
        |p| {
            if let Some(p) = is_image(&p) {
                all_files.push(p);
            } else {
                all_files.extend(filter_images(&p));
            }
        },
    );
    all_files.sort();

    std::fs::canonicalize(&args.output).map_or_else(
        |_| std::fs::create_dir_all(&args.output).expect("Unable to create output directory"),
        |p| {
            if !p.is_dir() {
                eprintln!("Output is not a directory");
                std::process::exit(1);
            }
        },
    );

    let trimmer = Trimmer {
        threshold: args.threshold,
        horizontal: args.horizontal,
    };
    all_files
        .par_iter()
        .for_each(|p| trimmer.trim(p, &args.output));
}
