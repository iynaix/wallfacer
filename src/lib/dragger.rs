use crate::{cropper::Direction, geometry::Geometry};

/// stores dragging state
#[derive(Debug, Clone, Default)]
pub struct Dragger {
    pub is_dragging: bool,
    pub x: f64,
    pub y: f64,
    pub preview_w: f64,
    pub preview_h: f64,
    pub image_w: f64,
    pub image_h: f64,
}

impl Dragger {
    pub fn new((image_w, image_h): (f64, f64), (preview_w, preview_h): (f64, f64)) -> Self {
        Self {
            preview_w,
            preview_h,
            image_w,
            image_h,
            ..Default::default()
        }
    }

    pub fn direction(&self, geom: &Geometry) -> Direction {
        if (self.image_h - f64::from(geom.h)).abs() < f64::EPSILON {
            Direction::X
        } else {
            Direction::Y
        }
    }

    pub fn overlay_styles(&self, geom: &Geometry) -> (String, String) {
        match self.direction(geom) {
            Direction::X => {
                let start = f64::from(geom.x) / self.image_w;
                let end = 1.0 - (f64::from(geom.x + geom.w) / self.image_w);

                (
                    format!("transform-origin: left; transform: scaleX({start})"),
                    format!("transform-origin: right; transform: scaleX({end})"),
                )
            }
            Direction::Y => {
                let start = f64::from(geom.y) / self.image_h;
                let end = 1.0 - (f64::from(geom.y + geom.h) / self.image_h);

                (
                    format!("transform-origin: top; transform: scaleY({start})"),
                    format!("transform-origin: bottom; transform: scaleY({end})"),
                )
            }
        }
    }

    pub fn update(&mut self, (new_x, new_y): (f64, f64), geom: &Geometry) -> Geometry {
        match self.direction(geom) {
            Direction::X => {
                let dx = self.image_w / self.preview_w * (new_x - self.x);
                Geometry {
                    x: (f64::from(geom.x) + dx).clamp(0.0, self.image_w - f64::from(geom.w)) as u32,
                    ..geom.clone()
                }
            }
            Direction::Y => {
                let dy = self.image_h / self.preview_h * (new_y - self.y);
                Geometry {
                    y: (f64::from(geom.y) + dy).clamp(0.0, self.image_h - f64::from(geom.h)) as u32,
                    ..geom.clone()
                }
            }
        }
    }
}
