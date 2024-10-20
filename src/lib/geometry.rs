use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeometryError {
    #[error("Invalid geometry coordinates")]
    InvalidCoordinate,
}

// hash used for deduping
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Geometry {
    pub w: u32,
    pub h: u32,
    pub x: u32,
    pub y: u32,
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}+{}+{}", self.w, self.h, self.x, self.y)
    }
}

impl std::cmp::Ord for Geometry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.x.cmp(&other.x).then_with(|| self.y.cmp(&other.y))
    }
}

impl std::cmp::PartialOrd for Geometry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<String> for Geometry {
    type Error = GeometryError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let geometry: Vec<_> = s
            .split(['x', '+'])
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();

        if geometry.len() != 4 {
            return Err(GeometryError::InvalidCoordinate);
        }

        Ok(Self {
            w: geometry[0],
            h: geometry[1],
            x: geometry[2],
            y: geometry[3],
        })
    }
}

impl Serialize for Geometry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Some(format!("{}+{}", self.x, self.y)).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Geometry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
    }
}

impl Geometry {
    #[must_use]
    pub fn align_start(&self, _img_width: u32, _img_height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn align_center(&self, img_width: u32, img_height: u32) -> Self {
        if img_height == self.h {
            Self {
                x: (img_width - self.w) / 2,
                y: 0,
                ..self.clone()
            }
        } else {
            Self {
                x: 0,
                y: (img_height - self.h) / 2,
                ..self.clone()
            }
        }
    }

    #[must_use]
    pub fn align_end(&self, img_width: u32, img_height: u32) -> Self {
        if img_height == self.h {
            Self {
                x: img_width - self.w,
                y: 0,
                ..self.clone()
            }
        } else {
            Self {
                x: 0,
                y: img_height - self.h,
                ..self.clone()
            }
        }
    }
}
