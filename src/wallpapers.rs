use indexmap::IndexMap;
use itertools::Itertools;
use serde::{
    de::{self},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::HashMap, path::PathBuf};

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
    pub const fn dir_bounds(&self, direction: Direction) -> (u32, u32) {
        match direction {
            Direction::X => (self.xmin, self.xmax),
            Direction::Y => (self.ymin, self.ymax),
        }
    }

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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WallInfo {
    pub filename: String,
    pub width: u32,
    pub height: u32,
    pub faces: Vec<Face>,
    pub geometries: HashMap<AspectRatio, Geometry>,
    pub wallust: String,
}

impl Serialize for WallInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(5 + self.geometries.len()))?;

        let (width, height) = image::image_dimensions(self.path())
            .unwrap_or_else(|_| panic!("could not open image: {:?}", self.path()));

        seq.serialize_element(&self.filename)?;
        seq.serialize_element(&width)?;
        seq.serialize_element(&height)?;
        seq.serialize_element(
            // serialize Vec<Face> as a json string
            &serde_json::to_string(&self.faces).expect("could not serialize faces"),
        )?;
        for (_, geom) in self.sorted_geometries() {
            seq.serialize_element(&geom)?;
        }
        seq.serialize_element(&self.wallust)?;

        seq.end()
    }
}

impl<'de> Deserialize<'de> for WallInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Filename,
            Faces,
            Geometries,
            Wallust,
        }

        struct WallInfoVisitor;

        impl<'de> de::Visitor<'de> for WallInfoVisitor {
            type Value = WallInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct WallInfo2")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut filename = None;
                let mut width = None;
                let mut height = None;
                let mut faces = None;
                let mut geometries: HashMap<AspectRatio, Geometry> = HashMap::new();
                let mut wallust = None;

                while let Some((key, value)) = map.next_entry::<&str, String>()? {
                    match key {
                        "filename" => {
                            filename = Some(value);
                        }
                        "width" => {
                            width = Some(value.parse::<u32>().map_err(de::Error::custom)?);
                        }
                        "height" => {
                            height = Some(value.parse::<u32>().map_err(de::Error::custom)?);
                        }
                        "faces" => {
                            faces =
                                Some(serde_json::from_str::<Vec<Face>>(&value).unwrap_or_else(
                                    |_| panic!("could not parse faces: {:?}", &value),
                                ));
                        }
                        "wallust" => {
                            wallust = Some(value);
                        }
                        _ => {
                            geometries.insert(
                                key.try_into().unwrap_or_else(|()| {
                                    panic!("could not convert aspect ratio {key} into string")
                                }),
                                value
                                    .try_into()
                                    .expect("could not convert geometry into string"),
                            );
                        }
                    }
                }

                let filename = filename.ok_or_else(|| de::Error::missing_field("filename"))?;
                let width = width.ok_or_else(|| de::Error::missing_field("width"))?;
                let height = height.ok_or_else(|| de::Error::missing_field("height"))?;
                let faces = faces.ok_or_else(|| de::Error::missing_field("faces"))?;
                let wallust = wallust.ok_or_else(|| de::Error::missing_field("wallust"))?;

                // geometries have no width and height, calculate from wall info
                let cropper = Cropper::new(&filename, &faces, width, height);
                let geometries = geometries
                    .iter()
                    .map(|(ratio, geom)| {
                        let (w, h, _) = cropper.crop_rect(ratio);
                        (
                            ratio.clone(),
                            Geometry {
                                w,
                                h,
                                ..geom.clone()
                            },
                        )
                    })
                    .collect();

                Ok(WallInfo {
                    filename,
                    width,
                    height,
                    faces,
                    geometries,
                    wallust,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "filename",
            "width",
            "height",
            "faces",
            "geometries",
            "wallust",
        ];
        deserializer.deserialize_struct("WallInfo", FIELDS, WallInfoVisitor)
    }
}

impl WallInfo {
    pub fn path(&self) -> PathBuf {
        wallpaper_dir().join(&self.filename)
    }

    pub const fn direction(&self, g: &Geometry) -> Direction {
        if self.height == g.h {
            Direction::X
        } else {
            Direction::Y
        }
    }

    pub fn cropper(&self) -> Cropper {
        Cropper::new(
            &filename(&self.path()),
            &self.faces,
            self.width,
            self.height,
        )
    }

    pub fn get_geometry(&self, ratio: &AspectRatio) -> Geometry {
        self.geometries
            .get(ratio)
            .map_or_else(|| self.cropper().crop(ratio), std::clone::Clone::clone)
    }

    pub fn set_geometry(&mut self, ratio: &AspectRatio, new_geom: &Geometry) {
        self.geometries.insert(ratio.clone(), new_geom.clone());
    }

    // sort geometries by key
    pub fn sorted_geometries(&self) -> impl Iterator<Item = (&AspectRatio, &Geometry)> {
        self.geometries.iter().sorted_by_key(|(k, _)| *k)
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
        let img_w = f64::from(self.width);
        let img_h = f64::from(self.height);

        if self.height == g.h {
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
    pub fn load() -> Self {
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

    pub fn header(&self) -> Vec<String> {
        let mut header: Vec<String> = vec![
            "filename".into(),
            "width".into(),
            "height".into(),
            "faces".into(),
        ];
        for (ratio, _) in self
            .wallpapers
            .iter()
            .next()
            .expect("could not get first column")
            .1
            .sorted_geometries()
        {
            header.push(ratio.to_string());
        }
        header.push("wallust".into());
        header
    }

    pub fn save(&self) {
        let writer = std::io::BufWriter::new(
            std::fs::File::create(&self.csv).expect("could not create wallpapers.csv"),
        );

        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(writer);

        // manually write the header
        wtr.write_record(self.header())
            .expect("could not write csv header");

        for row in self.wallpapers.values() {
            if wallpaper_dir().join(&row.filename).exists() {
                wtr.serialize(row).unwrap_or_else(|e| {
                    eprintln!("{:?}", e);
                    panic!("could not write row: {:?}", &row);
                });
            }
        }
    }
}

impl Default for WallpapersCsv {
    fn default() -> Self {
        Self::load()
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
