use indexmap::IndexMap;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

use crate::{
    cropper::{Cropper, Geometry},
    full_path, wallpaper_dir,
};

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq)]
pub struct Face {
    pub xmin: u32,
    pub xmax: u32,
    pub ymin: u32,
    pub ymax: u32,
}

impl Face {
    pub const fn width(&self) -> u32 {
        self.xmax - self.xmin
    }

    pub const fn height(&self) -> u32 {
        self.ymax - self.ymin
    }

    pub const fn area(&self) -> u32 {
        (self.xmax - self.xmin) * (self.ymax - self.ymin)
    }

    pub fn geometry(&self) -> String {
        format!(
            "{}x{}+{}+{}",
            self.width(),
            self.height(),
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
    #[serde(deserialize_with = "from_faces")]
    pub faces: Vec<Face>,
    pub r1440x2560: String,
    pub r2256x1504: String,
    pub r3440x1440: String,
    pub r1920x1080: String,
    pub r1x1: String,
    pub wallust: String,
}

impl WallInfo {
    pub fn path(&self) -> PathBuf {
        wallpaper_dir().join(&self.filename)
    }

    pub fn image_dimensions(&self) -> (u32, u32) {
        let img = image::open(self.path()).expect("could not open image");
        (img.width(), img.height())
    }

    pub fn cropper(&self) -> Cropper {
        Cropper::new(
            &self
                .path()
                .to_str()
                .expect("could not convert path to str")
                .to_string(),
            &self.faces,
        )
    }

    pub const fn get_geometry(&self, width: i32, height: i32) -> Option<&String> {
        match (width, height) {
            (1440, 2560) => Some(&self.r1440x2560),
            (2256, 1504) => Some(&self.r2256x1504),
            (3440, 1440) => Some(&self.r3440x1440),
            (1920, 1080) => Some(&self.r1920x1080),
            (1, 1) => Some(&self.r1x1),
            _ => None,
        }
    }

    pub fn overlay_percents(&self, g: &Geometry) -> (f32, f32) {
        let (img_width, _) = self.image_dimensions();

        (
            g.x as f32 / img_width as f32 * 100.0,
            (1.0 - (g.x + g.w) as f32 / img_width as f32) * 100.0,
        )
    }
}

pub struct Wallpapers {
    wallpapers: IndexMap<String, WallInfo>,
    csv: PathBuf,
}

impl Wallpapers {
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

impl Default for Wallpapers {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<&str> for Wallpapers {
    type Output = WallInfo;

    fn index(&self, index: &str) -> &Self::Output {
        &self.wallpapers[&index.to_string()]
    }
}

impl std::ops::Index<String> for Wallpapers {
    type Output = WallInfo;

    fn index(&self, index: String) -> &Self::Output {
        &self.wallpapers[&index]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct WallpapersIter<'a> {
    iter: indexmap::map::Iter<'a, String, WallInfo>,
}

impl<'a> Iterator for WallpapersIter<'a> {
    type Item = (&'a String, &'a WallInfo);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> IntoIterator for &'a Wallpapers {
    type Item = (&'a String, &'a WallInfo);
    type IntoIter = WallpapersIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        WallpapersIter {
            iter: self.wallpapers.iter(),
        }
    }
}
