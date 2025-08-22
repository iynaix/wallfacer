use indexmap::IndexMap;
use itertools::Itertools;
use rexiv2::Metadata;
use std::path::{Path, PathBuf};

use super::{
    aspect_ratio::AspectRatio,
    cropper::{Cropper, Direction},
    geometry::Geometry,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WallInfo {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub faces: Vec<Geometry>,
    pub scale: Option<u32>,
    pub geometries: IndexMap<AspectRatio, Geometry>,
    pub wallust: String,
}

impl WallInfo {
    pub fn new_from_file<P>(img: P) -> Self
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let (width, height) =
            image::image_dimensions(&img).expect("could not get image dimensions");

        let meta = Metadata::new_from_path(img.as_ref()).expect("could not init new metadata");

        let mut faces = Vec::new();
        let mut scale = None;
        let mut crops = IndexMap::new();
        let mut wallust = String::new();

        for tag in meta.get_xmp_tags().expect("unable to read xmp tags") {
            match tag.as_str() {
                "Xmp.wallfacer.faces" => {
                    let face_str = meta.get_tag_string(&tag).expect("could not get faces tag");

                    // empty faces are written as "[]" as rexiv2 seems to return the value of
                    // the next Xmp field, which is wrong
                    if face_str != "[]" {
                        faces = face_str
                            .split(',')
                            .map(|face| {
                                face.try_into().unwrap_or_else(|_| {
                                    panic!("could not convert face {face} into string")
                                })
                            })
                            .collect();
                    }
                }
                "Xmp.wallfacer.wallust" => {
                    wallust = meta
                        .get_tag_string(&tag)
                        .expect("could not get wallust tag");
                }
                "Xmp.wallfacer.scale" => {
                    scale = meta
                        .get_tag_string(&tag)
                        .expect("could not get scale tag")
                        .parse::<u32>()
                        .ok();
                }
                tag if tag.starts_with("Xmp.wallfacer.crop.") => {
                    let aspect = tag
                        .strip_prefix("Xmp.wallfacer.crop.")
                        .expect("could not strip cropdata prefix");
                    let aspect: AspectRatio = aspect
                        .try_into()
                        .unwrap_or_else(|_| panic!("could not parse aspect ratio {aspect}"));
                    let geom_str = meta.get_tag_string(tag).expect("could not get crop tag");
                    let geoms: Geometry = geom_str
                        .as_str()
                        .try_into()
                        .unwrap_or_else(|_| panic!("could not parse crop {geom_str}"));

                    crops.insert(aspect, geoms);
                }
                _ => {}
            }
        }

        Self {
            width,
            height,
            path: img.as_ref().to_path_buf(),
            scale,
            faces,
            geometries: crops,
            wallust,
        }
    }

    pub fn save(&self) -> rexiv2::Result<()> {
        let prev_modified = std::fs::metadata(&self.path)
            .and_then(|metadata| metadata.modified())
            .ok();

        let meta = Metadata::new_from_path(&self.path)?;

        // set face metadata
        let face_strings = if self.faces.is_empty() {
            "[]".to_string()
        } else {
            self.faces
                .iter()
                .map(std::string::ToString::to_string)
                .join(",")
        };

        meta.set_tag_string("Xmp.wallfacer.faces", &face_strings)?;

        if let Some(scale) = self.scale {
            meta.set_tag_string("Xmp.wallfacer.scale", &scale.to_string())?;
        }

        // set crop data
        for (aspect, geom) in &self.geometries {
            let crop_key = format!("Xmp.wallfacer.crop.{}", aspect);
            meta.set_tag_string(&crop_key, &geom.to_string())?;
        }

        if !self.wallust.is_empty() {
            meta.set_tag_string("Xmp.wallfacer.wallust", &self.wallust)?;
        }

        meta.save_to_file(&self.path)?;

        // reset the modified time to maintain sort order
        if let Some(prev_modified) = prev_modified {
            std::fs::OpenOptions::new()
                .write(true)
                .open(&self.path)
                .and_then(|f| f.set_modified(prev_modified))
                .ok();
        }

        Ok(())
    }

    pub fn get_target_scale(&self, min_width: u32, min_height: u32) -> Option<u32> {
        (1..=4).find(|&scale| self.width * scale >= min_width && self.height * scale >= min_height)
    }

    pub fn dimensions_f64(&self) -> (f64, f64) {
        (f64::from(self.width), f64::from(self.height))
    }

    pub fn has_metadata<P>(img: P) -> bool
    where
        P: AsRef<Path>,
    {
        Metadata::new_from_path(img.as_ref())
            .and_then(|meta| meta.get_tag_string("Xmp.wallfacer.faces"))
            .is_ok()
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

impl std::ops::Mul<u32> for WallInfo {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            width: self.width * rhs,
            height: self.height * rhs,
            faces: self.faces.into_iter().map(|face| face * rhs).collect(),
            geometries: self
                .geometries
                .into_iter()
                .map(|(ratio, geom)| (ratio, geom * rhs))
                .collect(),
            ..self
        }
    }
}
