use itertools::Itertools;
use thiserror::Error;

use crate::cropper::Direction;

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

impl std::ops::Mul<u32> for Geometry {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            w: self.w * rhs,
            h: self.h * rhs,
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl TryFrom<&str> for Geometry {
    type Error = GeometryError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let geometry = s
            .split(['x', '+'])
            .flat_map(str::parse::<u32>)
            .collect_vec();

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

impl Geometry {
    #[inline]
    pub const fn xmax(&self) -> u32 {
        self.x + self.w
    }

    #[inline]
    pub const fn ymax(&self) -> u32 {
        self.y + self.h
    }

    #[inline]
    pub const fn area(&self) -> u32 {
        self.w * self.h
    }

    pub const fn direction_bounds(&self, direction: Direction) -> (u32, u32) {
        match direction {
            Direction::X => (self.x, self.xmax()),
            Direction::Y => (self.y, self.ymax()),
        }
    }

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
