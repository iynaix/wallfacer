// trimming images

use std::path::PathBuf;

use image::{GenericImageView, ImageBuffer, ImageReader, Rgb};

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

fn is_trimmable<'a>(pixels: impl Iterator<Item = &'a Rgb<u8>>) -> bool {
    const THRESHOLD: f64 = 5.0;

    let mut reds: Vec<i32> = Vec::new();
    let mut greens: Vec<i32> = Vec::new();
    let mut blues: Vec<i32> = Vec::new();

    for px in pixels {
        reds.push(px[0].into());
        greens.push(px[1].into());
        blues.push(px[2].into());
    }

    let r_stddev = std_deviation(&reds);
    let g_stddev = std_deviation(&greens);
    let b_stddev = std_deviation(&blues);

    // println!("{r_stddev:?} {g_stddev:?} {b_stddev:?}");

    r_stddev < THRESHOLD && g_stddev < THRESHOLD && b_stddev < THRESHOLD
}

macro_rules! find_trimmable {
    ($outer_range:expr, $inner_range:expr, $get_pixel:expr) => {{
        let mut outer_range = $outer_range;
        let first = outer_range.next().expect("could not get first row / col");
        let second = outer_range.next().expect("could not get second row / col");

        // closure for checking row / colum
        let trimmable = |outer| {
            let pixels = $inner_range.map(|inner| $get_pixel(outer, inner));
            !is_trimmable(pixels)
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
    }};
}

fn trimmed_area(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> (u32, u32, u32, u32) {
    let width = img.width();
    let height = img.height();

    let y_start = find_trimmable!(0..height, 0..width, |y, x| img.get_pixel(x, y));
    // add 1 due to non-inclusive range
    let y_end = find_trimmable!((y_start..height).rev(), 0..width, |y, x| img
        .get_pixel(x, y))
        + 1;

    // TODO, option for horizontal trimming?

    /*
    // use y_start..y_end as those rows would already be cropped anyway
    let x_start = find_trimmable!(0..width, y_start..y_end, |x, y| img.get_pixel(x, y));
    // add 1 due to non-inclusive range
    let x_end = find_trimmable!((x_start..width).rev(), y_start..y_end, |x, y| img
        .get_pixel(x, y))
        + 1;
    */

    let x_start = 0;
    let x_end = width;

    // x, y, width, height
    (x_start, y_start, x_end - x_start, y_end - y_start)
}

fn trim_image(wall: &PathBuf) {
    println!("Processing: {wall:?}");

    let out = PathBuf::from("/home/iynaix/Pictures/trimmed")
        .join(wall.file_name().expect("could not get filename"));

    let img = ImageReader::open(wall)
        .expect("could not open image")
        .decode()
        .expect("could not decode image")
        .to_rgb8();

    let (x, y, width, height) = trimmed_area(&img);

    // nothing to trim
    if width == img.width() && height == img.height() {
        std::fs::copy(wall, out).expect("could not copy file");
        return;
    }

    // println!("{:?}", (x, y, width, height));
    // std::process::exit(1);
    let cropped = img.view(x, y, width, height).to_image();

    cropped
        .save(out)
        .unwrap_or_else(|_| panic!("could not save trimmed image for {wall:?}"));
}

fn main() {
    for entry in std::fs::read_dir("/home/iynaix/Pictures/trim").expect("could not read dir") {
        let wall = entry.expect("could not read entry").path();

        trim_image(&wall);
    }

    // trim_image(&PathBuf::from("/home/iynaix/Pictures/trim/97693879_p0.jpg"));
    // trim_image(&PathBuf::from("/home/iynaix/Pictures/trim/64337772_p0.jpg"));

    // trim_image(&PathBuf::from("/home/iynaix/Pictures/trim/71633610_p0.jpg"));

    // super problematic
    // trim_image(&PathBuf::from("/home/iynaix/Pictures/trim/71820780_p0.jpg"));
}
