use itertools::Itertools;
use std::path::PathBuf;

use wallfacer::{
    aspect_ratio::AspectRatio, config::ConfigResolution, cropper::Direction, geometry::Geometry,
    wallpapers::WallInfo,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wall {
    /// the original wallinfo before any modifications
    pub source: WallInfo,
    pub current: WallInfo,
    /// currently selected ratio
    pub ratio: AspectRatio,
    path: PathBuf,
    /// possible ratios for this image
    pub ratios: Vec<ConfigResolution>,
    pub mouseover_geom: Option<Geometry>,
}

impl Wall {
    pub fn new(info: &WallInfo, path: PathBuf, resolutions: &[ConfigResolution]) -> Self {
        let ratios = resolutions
            .iter()
            .filter(|res| {
                const THRESHOLD: f64 = 1.0 / 100.0;

                // don't show resolution if aspect ratio of image is within a percentage threshold
                let ratio = f64::from(&res.resolution);

                let (w, h) = info.dimensions_f64();
                let lower_w = (w * (1.0 - THRESHOLD)) / h;
                let upper_w = (w * (1.0 + THRESHOLD)) / h;

                if lower_w <= ratio && ratio <= upper_w {
                    return false;
                }

                let lower_h = w / (h * (1.0 - THRESHOLD));
                let upper_h = w / (h * (1.0 + THRESHOLD));

                if lower_h <= ratio && ratio <= upper_h {
                    return false;
                }

                true
            })
            .cloned()
            .collect_vec();

        Self {
            source: info.clone(),
            current: info.clone(),
            path,
            ratio: ratios
                .first()
                .expect("no resolutions provided")
                .resolution
                .clone(),
            ratios,
            mouseover_geom: None,
        }
    }

    /// needs to be a string to be passed as an <img> src
    pub fn path(&self) -> &str {
        self.path
            .to_str()
            .unwrap_or_else(|| panic!("could not convert {} to str", self.path.display()))
    }

    /// gets geometry for current aspect ratio
    pub fn get_geometry(&self) -> Geometry {
        self.current.get_geometry(&self.ratio)
    }

    /// sets the geometry for current aspect ratio
    pub fn set_geometry(&mut self, geom: &Geometry) {
        self.current.set_geometry(&self.ratio, geom);
    }

    /// moves the crop area of the current wallpaper based on its direction
    pub fn move_geometry_by(&self, delta: f64) -> Geometry {
        let current_geom = self.get_geometry();

        let negative_delta = delta.is_sign_negative();
        let delta = (if negative_delta { -delta } else { delta }) as u32;

        match self.current.direction(&current_geom) {
            Direction::X => Geometry {
                x: if negative_delta {
                    current_geom.x.max(delta) - delta
                } else {
                    (current_geom.x + delta).min(self.current.width - current_geom.w)
                },
                ..current_geom
            },
            Direction::Y => Geometry {
                y: if negative_delta {
                    current_geom.y.max(delta) - delta
                } else {
                    (current_geom.y + delta).min(self.current.height - current_geom.h)
                },
                ..current_geom
            },
        }
    }

    /// centers the geometry on a face
    pub fn center_on_face(&self, face: &Geometry) -> Geometry {
        let geom = self.get_geometry();
        let direction = if self.current.height == geom.h {
            Direction::X
        } else {
            Direction::Y
        };

        self.current
            .cropper()
            .crop_single_face(face, direction, geom.w, geom.h)
    }
}
