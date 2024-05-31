use serde::Serialize;

/// euclid's algorithm to find the greatest common divisor
const fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AspectRatio {
    pub w: u32,
    pub h: u32,
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
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split('x').collect();
        assert!(parts.len() == 2, "Invalid aspect ratio: {}", s);

        let width = parts[0].parse().map_err(|_| ())?;
        let height = parts[1].parse().map_err(|_| ())?;

        Ok(Self::new(width, height))
    }
}

impl Serialize for AspectRatio {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
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
