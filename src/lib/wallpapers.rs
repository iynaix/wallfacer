use indexmap::IndexMap;
use serde::{
    de::{self},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::HashMap, path::Path};

use crate::filename;

use super::{
    aspect_ratio::AspectRatio,
    config::WallpaperConfig,
    cropper::{Cropper, Direction},
    geometry::Geometry,
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
                                key.try_into().unwrap_or_else(|_| {
                                    panic!("could not convert aspect ratio {key} into string")
                                }),
                                value
                                    .try_into()
                                    .expect("could not convert geometry into string"),
                            );
                        }
                    }
                }

                Ok(WallInfo {
                    filename: filename.ok_or_else(|| de::Error::missing_field("filename"))?,
                    width: width.ok_or_else(|| de::Error::missing_field("width"))?,
                    height: height.ok_or_else(|| de::Error::missing_field("height"))?,
                    faces: faces.ok_or_else(|| de::Error::missing_field("faces"))?,
                    wallust: wallust.ok_or_else(|| de::Error::missing_field("wallust"))?,
                    geometries,
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
    pub fn dimensions_f64(&self) -> (f64, f64) {
        (f64::from(self.width), f64::from(self.height))
    }

    pub fn ratio(&self) -> f64 {
        f64::from(self.width) / f64::from(self.height)
    }

    pub const fn direction(&self, g: &Geometry) -> Direction {
        if self.height == g.h {
            Direction::X
        } else {
            Direction::Y
        }
    }

    pub fn cropper(&self) -> Cropper {
        Cropper::new(&self.faces, self.width, self.height)
    }

    pub fn get_geometry(&self, ratio: &AspectRatio) -> Geometry {
        self.geometries
            .get(ratio)
            .map_or_else(|| self.cropper().crop(ratio), std::clone::Clone::clone)
    }

    pub fn set_geometry(&mut self, ratio: &AspectRatio, new_geom: &Geometry) {
        self.geometries.insert(ratio.clone(), new_geom.clone());
    }

    pub fn is_default_crops(&self, resolutions: &[AspectRatio]) -> bool {
        let cropper = self.cropper();

        resolutions
            .iter()
            .all(|ratio| self.get_geometry(ratio) == cropper.crop(ratio))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpapersCsv {
    wallpapers: IndexMap<String, WallInfo>,
    config: WallpaperConfig,
}

impl WallpapersCsv {
    pub fn open(config: &WallpaperConfig) -> Result<Self, std::io::Error> {
        std::fs::File::open(&config.csv_path).map(|csv_file| {
            let mut reader = csv::Reader::from_reader(std::io::BufReader::new(csv_file));

            Self {
                config: config.clone(),
                wallpapers: reader
                    .deserialize::<WallInfo>()
                    .flatten()
                    .map(|wall_info| (wall_info.filename.to_string(), wall_info))
                    .collect(),
            }
        })
    }

    pub fn load(config: &WallpaperConfig) -> Self {
        Self::open(config).unwrap_or_else(|_| {
            eprintln!("wallpapers.csv not found! Have you run \"wallpapers-add\" to create it?");
            std::process::exit(1);
        })
    }

    pub fn find_duplicates(&self) {
        let mut groups: HashMap<_, Vec<_>> = HashMap::new();

        // check for duplicates using the faces array
        for wall_info in self.wallpapers.values().filter(|w| !w.faces.is_empty()) {
            groups
                .entry(format!(
                    "{}|{}|{:?}",
                    &wall_info.width, &wall_info.height, &wall_info.faces
                ))
                .or_default()
                .push(wall_info);
        }

        let duplicates: Vec<_> = groups
            .into_iter()
            .filter_map(|(_, v)| if v.len() > 1 { Some(v) } else { None })
            .collect();

        if !duplicates.is_empty() {
            for infos in duplicates {
                eprintln!("Duplicate faces:");
                for info in infos {
                    eprintln!("  {:?}", info.filename);
                }
                eprintln!();
            }

            std::process::exit(1);
        }
    }

    pub fn iter(&self) -> WallpapersIter {
        WallpapersIter {
            iter: self.wallpapers.iter(),
        }
    }

    pub fn get<P>(&self, fname: P) -> Option<&WallInfo>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        self.wallpapers.get(&filename(fname))
    }

    pub fn insert(&mut self, wall_info: WallInfo) {
        self.wallpapers
            .insert(wall_info.filename.clone(), wall_info);
    }

    pub fn header(&self, ratios: &[AspectRatio]) -> Vec<String> {
        let mut header: Vec<String> = vec![
            "filename".into(),
            "width".into(),
            "height".into(),
            "faces".into(),
        ];
        header.extend(ratios.iter().map(std::string::ToString::to_string));
        header.push("wallust".into());
        header
    }

    pub fn save(&mut self, ratios: &[AspectRatio]) {
        let writer = std::io::BufWriter::new(
            std::fs::File::create(&self.config.csv_path).expect("could not create wallpapers.csv"),
        );

        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(writer);

        // manually write the header
        wtr.write_record(self.header(ratios))
            .expect("could not write csv header");

        let mut removed = Vec::new();
        for wall in self.wallpapers.values() {
            let wall_path = self.config.wallpapers_dir.join(&wall.filename);
            if wall_path.exists() {
                let (width, height) = image::image_dimensions(&wall_path)
                    .unwrap_or_else(|_| panic!("could not open image: {:?}", &wall_path));
                let mut record: Vec<String> = vec![
                    wall.filename.to_string(),
                    width.to_string(),
                    height.to_string(),
                    serde_json::to_string(&wall.faces).expect("could not serialize faces"),
                ];
                for resolution in ratios {
                    record.push(wall.get_geometry(resolution).to_string());
                }
                record.push(wall.wallust.to_string());

                wtr.write_record(record).unwrap_or_else(|e| {
                    eprintln!("{:?}", e);
                    panic!("could not write row: {:?}", &wall);
                });
            } else {
                removed.push(wall.filename.to_string());
                println!("Removed wallpaper: {}", wall.filename);
            }
        }

        for r in removed {
            self.wallpapers.shift_remove(&r);
        }
    }
}

impl Default for WallpapersCsv {
    fn default() -> Self {
        Self {
            wallpapers: IndexMap::new(),
            config: WallpaperConfig::new(),
        }
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
