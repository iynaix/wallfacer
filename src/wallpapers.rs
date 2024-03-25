use indexmap::IndexMap;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

use crate::{
    cropper::{AspectRatio, Cropper, Direction},
    filename, full_path,
    geometry::Geometry,
    wallpaper_dir,
};

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq)]
pub struct Face {
    pub xmin: u32,
    pub xmax: u32,
    pub ymin: u32,
    pub ymax: u32,
}

impl Face {
    #[inline]
    pub const fn area(&self) -> u32 {
        (self.xmax - self.xmin) * (self.ymax - self.ymin)
    }

    pub const fn geometry(&self) -> Geometry {
        Geometry {
            w: self.xmax - self.xmin,
            h: self.ymax - self.ymin,
            x: self.xmin,
            y: self.ymin,
        }
    }

    pub fn geometry_str(&self) -> String {
        format!(
            "{}x{}+{}+{}",
            self.xmax - self.xmin,
            self.ymax - self.ymin,
            self.xmin,
            self.ymin
        )
    }
}

impl Serialize for Face {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // required for setting order
        Some(vec![self.xmin, self.xmax, self.ymin, self.ymax]).serialize(serializer)
    }
}

// serialize Vec<Face> as a json string
fn to_faces<S>(faces: &Vec<Face>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = serde_json::to_string(faces).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&s)
}

// deserialize as a json string into a Vec<Face>
fn from_faces<'de, D>(deserializer: D) -> Result<Vec<Face>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct WallInfo {
    pub filename: String,
    #[serde(serialize_with = "to_faces", deserialize_with = "from_faces")]
    pub faces: Vec<Face>,
    pub r1440x2560: Geometry,
    pub r2256x1504: Geometry,
    pub r3440x1440: Geometry,
    pub r1920x1080: Geometry,
    pub r1x1: Geometry,
    pub wallust: String,
}

impl WallInfo {
    pub fn path(&self) -> PathBuf {
        wallpaper_dir().join(&self.filename)
    }

    #[inline]
    pub fn image_dimensions(&self) -> (u32, u32) {
        image::image_dimensions(self.path()).expect("could not open image")
    }

    #[inline]
    pub fn image_dimensions_f64(&self) -> (f64, f64) {
        let (w, h) = image::image_dimensions(self.path()).expect("could not open image");
        (f64::from(w), f64::from(h))
    }

    pub fn direction(&self, g: &Geometry) -> Direction {
        let (_, img_h) = self.image_dimensions();
        if img_h == g.h {
            Direction::X
        } else {
            Direction::Y
        }
    }

    pub fn cropper(&self) -> Cropper {
        Cropper::new(&filename(&self.path()), &self.faces)
    }

    pub fn get_geometry(&self, ratio: &AspectRatio) -> Geometry {
        match ratio {
            AspectRatio(1440, 2560) => self.r1440x2560.clone(),
            AspectRatio(2256, 1504) => self.r2256x1504.clone(),
            AspectRatio(3440, 1440) => self.r3440x1440.clone(),
            AspectRatio(1920, 1080) => self.r1920x1080.clone(),
            AspectRatio(1, 1) => self.r1x1.clone(),
            _ => self.cropper().crop(ratio),
        }
    }

    pub fn set_geometry(&mut self, ratio: &AspectRatio, new_geom: &Geometry) {
        match ratio {
            AspectRatio(1440, 2560) => self.r1440x2560 = new_geom.clone(),
            AspectRatio(2256, 1504) => self.r2256x1504 = new_geom.clone(),
            AspectRatio(3440, 1440) => self.r3440x1440 = new_geom.clone(),
            AspectRatio(1920, 1080) => self.r1920x1080 = new_geom.clone(),
            AspectRatio(1, 1) => self.r1x1 = new_geom.clone(),
            _ => {}
        }
    }

    pub fn is_default_crops(&self) -> bool {
        let cropper = self.cropper();

        for ratio in [
            AspectRatio(1440, 2560),
            AspectRatio(2256, 1504),
            AspectRatio(3440, 1440),
            AspectRatio(1920, 1080),
            AspectRatio(1, 1),
        ] {
            if self.get_geometry(&ratio) != cropper.crop(&ratio) {
                return false;
            }
        }
        true
    }

    pub fn overlay_transforms(&self, g: &Geometry) -> (Direction, f64, f64) {
        let (img_w, img_h) = self.image_dimensions_f64();

        if img_h as u32 == g.h {
            (
                Direction::X,
                f64::from(g.x) / img_w,
                (1.0 - f64::from(g.x + g.w) / img_w),
            )
        } else {
            (
                Direction::Y,
                f64::from(g.y) / img_h,
                (1.0 - f64::from(g.y + g.h) / img_h),
            )
        }
    }
}

pub struct WallpapersCsv {
    wallpapers: IndexMap<String, WallInfo>,
    csv: PathBuf,
}

impl WallpapersCsv {
    pub fn new() -> Self {
        let wallpapers_csv = full_path("~/Pictures/Wallpapers/wallpapers.csv");

        let reader = std::io::BufReader::new(
            std::fs::File::open(&wallpapers_csv).expect("could not open wallpapers.csv"),
        );

        let mut rdr = csv::Reader::from_reader(reader);

        Self {
            csv: wallpapers_csv,
            wallpapers: rdr
                .deserialize::<WallInfo>()
                .flatten()
                .map(|wall_info| (wall_info.filename.to_string(), wall_info))
                .collect(),
        }
    }

    pub fn get(&self, filename: &str) -> Option<&WallInfo> {
        self.wallpapers.get(filename)
    }

    pub fn iter(&self) -> WallpapersIter {
        WallpapersIter {
            iter: self.wallpapers.iter(),
        }
    }

    pub fn insert(&mut self, filename: String, wall_info: WallInfo) {
        self.wallpapers.insert(filename, wall_info);
    }

    pub fn save(&self) {
        let writer = std::io::BufWriter::new(
            std::fs::File::create(&self.csv).expect("could not create wallpapers.csv"),
        );

        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for row in self.wallpapers.values() {
            if wallpaper_dir().join(&row.filename).exists() {
                wtr.serialize(row)
                    .unwrap_or_else(|_| panic!("could not write row: {:?}", &row));
            }
        }
    }
}

impl Default for WallpapersCsv {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WallpapersIter<'a> {
    iter: indexmap::map::Iter<'a, String, WallInfo>,
}

impl<'a> Iterator for WallpapersIter<'a> {
    type Item = (&'a String, &'a WallInfo);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> IntoIterator for &'a WallpapersCsv {
    type Item = (&'a String, &'a WallInfo);
    type IntoIter = WallpapersIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        WallpapersIter {
            iter: self.wallpapers.iter(),
        }
    }
}
