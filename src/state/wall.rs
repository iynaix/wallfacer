use indexmap::IndexMap;
use itertools::Itertools;
use std::path::PathBuf;

use wallfacer::{
    aspect_ratio::AspectRatio, cropper::Direction, geometry::Geometry, wallpapers::WallInfo,
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
    pub ratios: IndexMap<String, AspectRatio>,
}

impl Wall {
    pub fn new(
        info: &WallInfo,
        path: PathBuf,
        resolutions: &IndexMap<String, AspectRatio>,
    ) -> Self {
        let ratios: IndexMap<_, _> = resolutions
            .into_iter()
            .filter(|(_, ratio)| {
                // do not show resolution if aspect ratio of image is the same,
                // as there is only a single possible crop

                // TODO: don't show if it is within 5 pixels?
                (info.ratio() - f64::from(*ratio)).abs() > f64::EPSILON
            })
            .map(|(name, ratio)| (name.clone(), ratio.clone()))
            .collect();

        // TODO: reuse ratio from previous wallpaper?

        Self {
            source: info.clone(),
            current: info.clone(),
            path,
            ratio: ratios.first().expect("no resolutions provided").1.clone(),
            ratios,
        }
    }

    /// needs to be a string to be passed as an <img> src
    pub fn path(&self) -> &str {
        self.path
            .to_str()
            .unwrap_or_else(|| panic!("could not convert {:?} to str", self.path))
    }

    /// gets geometry for current aspect ratio
    pub fn get_geometry(&self) -> Geometry {
        self.current.get_geometry(&self.ratio)
    }

    /// sets the geometry for current aspect ratio
    pub fn set_geometry(&mut self, geom: &Geometry) {
        self.current.set_geometry(&self.ratio, geom);
    }

    /// unique crop candidates, suitable for candidate buttons
    pub fn candidate_geometries(&self) -> Vec<Geometry> {
        self.current
            .cropper()
            .crop_candidates(&self.ratio)
            .into_iter()
            .unique()
            .collect()
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
}
