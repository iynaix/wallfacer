use itertools::Itertools;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// euclid's algorithm to find the greatest common divisor
const fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}

#[derive(Error, Debug)]
pub enum AspectRatioError {
    #[error("Invalid aspect ratio")]
    InvalidAspectRatio,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AspectRatio {
    pub w: u32,
    pub h: u32,
}

impl<'de> Deserialize<'de> for AspectRatio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // Create a visitor to handle the deserialization
        struct AspectRatioVisitor;

        impl Visitor<'_> for AspectRatioVisitor {
            type Value = AspectRatio;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string in the format 'WxH'")
            }

            fn visit_str<E>(self, value: &str) -> Result<AspectRatio, E>
            where
                E: de::Error,
            {
                AspectRatio::try_from(value).map_err(de::Error::custom)
            }
        }

        // Use the visitor to deserialize
        deserializer.deserialize_str(AspectRatioVisitor)
    }
}

impl Serialize for AspectRatio {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl std::fmt::Display for AspectRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.w, self.h)
    }
}

impl PartialOrd for AspectRatio {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AspectRatio {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        f64::from(self)
            .partial_cmp(&f64::from(other))
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl From<&AspectRatio> for f64 {
    fn from(val: &AspectRatio) -> Self {
        Self::from(val.w) / Self::from(val.h)
    }
}

impl TryFrom<&str> for AspectRatio {
    type Error = AspectRatioError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts = s.split('x').flat_map(str::parse::<u32>).collect_vec();

        if parts.len() != 2 {
            return Err(AspectRatioError::InvalidAspectRatio);
        }

        Ok(Self::new(parts[0], parts[1]))
    }
}

impl AspectRatio {
    pub const fn new(width: u32, height: u32) -> Self {
        let divisor = gcd(width, height);
        Self {
            w: width / divisor,
            h: height / divisor,
        }
    }
}
