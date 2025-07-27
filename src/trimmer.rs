// trimming images

use std::path::PathBuf;

use image::{GenericImageView, ImageBuffer, ImageReader, Rgb};
use rayon::prelude::*;
use wallfacer::{PathBufNumericSort, cli::TrimmerArgs, filename, filter_images, is_image};

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

    fn trim(&self, wall: &PathBuf) {
        let img = ImageReader::open(wall)
            .expect("could not open image")
            .decode()
            .expect("could not decode image")
            .to_rgb8();

        let (x, y, width, height) = self.trimmed_area(&img);

        // nothing to trim
        if width == img.width() && height == img.height() {
            return;
        }

        let trimmed_fname = filename(wall).replace("jpeg", "jpg").replace("jpg", "png");
        let tmp_file = PathBuf::from("/tmp").join(&trimmed_fname);

        let cropped = img.view(x, y, width, height).to_image();
        cropped
            .save(&tmp_file)
            .unwrap_or_else(|_| panic!("could not save trimmed image for {}", wall.display()));

        std::thread::sleep(std::time::Duration::from_secs(1));

        // replace original file if it is not a jpeg
        let final_path = wall.with_file_name(&trimmed_fname);
        std::fs::copy(&tmp_file, &final_path).unwrap_or_else(|_| {
            panic!(
                "could not copy {} to {}",
                tmp_file.display(),
                final_path.display()
            )
        });

        if filename(wall) != trimmed_fname {
            std::fs::remove_file(wall).unwrap_or_else(|e| {
                eprintln!("{e}");
                panic!("could not remove {}", wall.display());
            });
        }
    }
}

pub fn main(args: &TrimmerArgs) {
    let mut all_files = Vec::new();
    std::fs::canonicalize(&args.path).map_or_else(
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
    all_files.numeric_sort();

    let trimmer = Trimmer {
        threshold: args.threshold,
        horizontal: args.horizontal,
    };
    all_files.par_iter().for_each(|wall| {
        println!("Processing: {}", wall.display());

        if !args.dry_run {
            trimmer.trim(wall);
        }
    });
}
