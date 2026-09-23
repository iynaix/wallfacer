use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use xmpkit::{XmpFile, XmpMeta, XmpResult, XmpValue, register_namespace};

use super::{
    aspect_ratio::AspectRatio,
    cropper::{Cropper, Direction},
    geometry::Geometry,
};

static WALLFACER_NS: &str = "http://example.com/wallfacer/";

pub fn save_preserving_modified<MetaFn>(path: &PathBuf, meta_fn: MetaFn) -> XmpResult<()>
where
    MetaFn: Fn(&str, &mut XmpMeta) -> XmpResult<XmpMeta>,
{
    let prev_modified = std::fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .ok();

    let mut fp = XmpFile::new();
    fp.open_with(path, xmpkit::XmpOptions::default().for_update())
        .expect("failed to open image");

    register_namespace(WALLFACER_NS, "wallfacer")?;

    let mut new_meta = XmpMeta::new();
    let meta = fp.get_xmp_mut().unwrap_or(&mut new_meta);
    let meta = meta_fn(WALLFACER_NS, meta)?;

    fp.put_xmp(meta);
    fp.save(path)?;

    // reset the modified time to maintain sort order
    if let Some(prev_modified) = prev_modified {
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .and_then(|f| f.set_modified(prev_modified))
            .ok();
    }

    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WallInfo {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub faces: Vec<Geometry>,
    pub scale: Option<u32>,
    pub geometries: IndexMap<AspectRatio, Geometry>,
}

impl WallInfo {
    pub fn new_from_file<P>(img: P) -> Self
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let (width, height) =
            image::image_dimensions(&img).expect("could not get image dimensions");

        let mut fp = XmpFile::new();
        fp.open(&img).expect("failed to open image");

        register_namespace(WALLFACER_NS, "wallfacer").expect("could not register namespace");

        let mut ret = Self {
            width,
            height,
            path: img.as_ref().to_path_buf(),
            ..Default::default()
        };

        if let Some(xmp) = fp.get_xmp() {
            if let Some(XmpValue::String(scale)) = xmp.get_property(WALLFACER_NS, "scale") {
                ret.scale = scale.parse().ok();
            }

            if let Some(XmpValue::Array(faces)) = xmp.get_property(WALLFACER_NS, "faces") {
                ret.faces = faces
                    .iter()
                    .map(|face| {
                        face.as_str()
                            .expect("could not convert face to str")
                            .try_into()
                            .unwrap_or_else(|_| panic!("could not convert face {face} into string"))
                    })
                    .collect();
            }

            if let Some(XmpValue::Structure(crops)) = xmp.get_property("wallfacer", "crops") {
                ret.geometries = crops
                    .iter()
                    .map(|(aspect, geom)| {
                        let aspect: AspectRatio = aspect
                            .as_str()
                            .strip_prefix(&format!("{WALLFACER_NS}:"))
                            .expect("cannot strip prefix")
                            .try_into()
                            .unwrap_or_else(|_| panic!("could not parse aspect ratio {aspect}"));

                        let geom: Geometry = geom
                            .as_str()
                            .expect("could not convert crop to str")
                            .try_into()
                            .unwrap_or_else(|_| {
                                panic!("could not convert crop {geom} into string")
                            });

                        (aspect, geom)
                    })
                    .collect();
            }
        } else {
            panic!("unable to read xmp metadata for {img:?}");
        }

        ret
    }

    pub fn save(&self) -> XmpResult<()> {
        save_preserving_modified(&self.path, |ns, _| {
            let mut meta = XmpMeta::new();

            if let Some(scale) = self.scale {
                meta.set_property(ns, "scale", XmpValue::Integer(scale.into()))?;
            };

            // set faces data
            for face in &self.faces {
                meta.append_array_item(WALLFACER_NS, "faces", XmpValue::String(face.to_string()))?;
            }

            // set crop data
            for (aspect, geom) in &self.geometries {
                meta.set_struct_field(
                    WALLFACER_NS,
                    "crops",
                    &aspect.to_string(),
                    XmpValue::String(geom.to_string()),
                )?;
            }

            Ok(meta)
        })
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
        let mut fp = XmpFile::new();
        fp.open(&img).expect("failed to open image");

        if let Some(xmp) = fp.get_xmp() {
            for prop in xmp.all_properties() {
                if prop.namespace_uri.contains("wallfacer") {
                    return true;
                }
            }
        }

        return false;
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
