use core::panic;
use itertools::Itertools;
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use thiserror::Error;

use image::image_dimensions;

use crate::{wallpaper_dir, wallpapers::Face};

#[derive(Debug, Clone)]
pub enum Direction {
    X,
    Y,
}

struct FaceInfo {
    area: u32,
    start: u32,
}

#[derive(Error, Debug)]
pub enum GeometryError {
    #[error("Invalid geometry string")]
    InvalidGeometry,
    #[error("Invalid geometry coordinates")]
    InvalidCoordinate,
}

#[derive(Debug, Default, Clone, PartialEq)]
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

impl std::convert::From<&String> for Geometry {
    fn from(s: &String) -> Self {
        let parts: Vec<&str> = s.split(['x', '+'].as_ref()).collect();
        if parts.len() != 4 {
            panic!("Invalid geometry string");
        }

        Geometry {
            w: parts[0].parse().expect("Invalid geometry coordinate"),
            h: parts[1].parse().expect("Invalid geometry coordinate"),
            x: parts[2].parse().expect("Invalid geometry coordinate"),
            y: parts[3].parse().expect("Invalid geometry coordinate"),
        }
    }
}

impl FromStr for Geometry {
    type Err = GeometryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(['x', '+'].as_ref()).collect();
        if parts.len() != 4 {
            return Err(GeometryError::InvalidGeometry);
        }

        Ok(Geometry {
            w: parts[0]
                .parse()
                .map_err(|_| GeometryError::InvalidCoordinate)?,
            h: parts[1]
                .parse()
                .map_err(|_| GeometryError::InvalidCoordinate)?,
            x: parts[2]
                .parse()
                .map_err(|_| GeometryError::InvalidCoordinate)?,
            y: parts[3]
                .parse()
                .map_err(|_| GeometryError::InvalidCoordinate)?,
        })
    }
}

pub struct Cropper {
    pub faces: Vec<Face>,
    pub image: PathBuf,
    pub width: u32,
    pub height: u32,
}

pub struct AspectRatio(u32, u32);

impl std::fmt::Display for AspectRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.0, self.1)
    }
}

pub const HD_RATIO: AspectRatio = AspectRatio(1920, 1080);
pub const ULTRAWIDE_RATIO: AspectRatio = AspectRatio(3440, 1440);
pub const VERTICAL_RATIO: AspectRatio = AspectRatio(1440, 2560);
pub const FRAMEWORK_RATIO: AspectRatio = AspectRatio(2256, 1504);
pub const SQUARE_RATIO: AspectRatio = AspectRatio(1, 1);

impl Cropper {
    pub fn new(image: &String, faces: &[Face]) -> Self {
        let image = wallpaper_dir().join(image);
        let (width, height) = image_dimensions(&image).expect("Failed to read image dimensions");

        Self {
            faces: faces.to_vec(),
            image,
            width,
            height,
        }
    }

    fn crop_rect(&self, aspect_ratio: &AspectRatio) -> (u32, u32, Direction) {
        let AspectRatio(target_w, target_h) = aspect_ratio;

        // Calculate width and height that can be cropped while maintaining aspect ratio
        let crop_w = std::cmp::min(self.width, self.height * target_w / target_h);
        let crop_h = std::cmp::min(self.height, self.width * target_h / target_w);

        // Choose the larger dimension to get the largest possible cropped rectangle
        let (crop_w, crop_h) = if crop_w * target_h > crop_h * target_w {
            (crop_w, crop_h)
        } else {
            (crop_h * target_w / target_h, crop_h)
        };

        (
            crop_w,
            crop_h,
            if crop_w == self.width {
                Direction::Y
            } else {
                Direction::X
            },
        )
    }

    fn clamp(
        &self,
        val: f32,
        direction: &Direction,
        target_width: u32,
        target_height: u32,
    ) -> Face {
        let min_ = val;
        match direction {
            Direction::X => {
                let max_ = min_ + target_width as f32;
                if min_ < 0.0 {
                    Face {
                        xmax: target_width,
                        ymax: self.height,
                        ..Face::default()
                    }
                } else if max_ > self.width as f32 {
                    Face {
                        xmin: self.width - target_width,
                        xmax: self.width,
                        ymax: self.height,
                        ..Face::default()
                    }
                } else {
                    Face {
                        xmin: min_ as u32,
                        xmax: max_ as u32,
                        ymax: self.height,
                        ..Face::default()
                    }
                }
            }
            Direction::Y => {
                let max_ = min_ + target_height as f32;
                if min_ < 0.0 {
                    Face {
                        ymax: target_height,
                        xmax: self.width,
                        ..Face::default()
                    }
                } else if max_ > self.height as f32 {
                    Face {
                        ymin: self.height - target_height,
                        ymax: self.height,
                        xmax: self.width,
                        ..Face::default()
                    }
                } else {
                    Face {
                        ymin: min_ as u32,
                        ymax: max_ as u32,
                        xmax: self.width,
                        ..Face::default()
                    }
                }
            }
        }
    }

    fn crop_single_face(
        &self,
        direction: &Direction,
        target_width: u32,
        target_height: u32,
    ) -> Face {
        let Face {
            xmin,
            xmax,
            ymin,
            ymax,
        } = &self.faces[0];
        match direction {
            Direction::X => {
                let mid = (xmin + xmax) as f32 / 2.0;
                self.clamp(
                    mid - (target_width as f32 / 2.0),
                    direction,
                    target_width,
                    target_height,
                )
            }
            Direction::Y => {
                let mid = (ymin + ymax) as f32 / 2.0;
                self.clamp(
                    mid - (target_height as f32 / 2.0),
                    direction,
                    target_width,
                    target_height,
                )
            }
        }
    }

    pub fn crop(&mut self, aspect_ratio: &AspectRatio) -> Face {
        let (target_width, target_height, direction) = self.crop_rect(aspect_ratio);

        if self.width == target_width && self.height == target_height {
            return Face {
                xmax: self.width,
                ymax: self.height,
                ..Face::default()
            };
        }

        if self.faces.is_empty() {
            return match direction {
                Direction::X => Face {
                    xmin: (self.width - target_width) / 2,
                    xmax: (self.width + target_width) / 2,
                    ymax: self.height,
                    ..Face::default()
                },
                Direction::Y => Face {
                    ymin: (self.height - target_height) / 2,
                    ymax: (self.height + target_height) / 2,
                    xmax: self.width,
                    ..Face::default()
                },
            };
        }

        if self.faces.len() == 1 {
            return self.crop_single_face(&direction, target_width, target_height);
        }

        // handle multiple faces
        self.faces.sort_by_key(|face| match direction {
            Direction::X => face.xmin,
            Direction::Y => face.ymin,
        });

        let mut max_faces = 0.0;
        let mut faces_info: Vec<FaceInfo> = vec![];
        let (rect_max, rect_len) = match direction {
            Direction::X => (self.width - target_width, target_width),
            Direction::Y => (self.height - target_height, target_height),
        };

        for rect_start in 0..rect_max {
            let rect_end = rect_start + rect_len;
            let mut num_faces: f32 = 0.0;
            let mut faces_area = 0;

            for face in &self.faces {
                // check number of faces in decimal within enclosed within larger rectangle
                let (min_, max_) = match direction {
                    Direction::X => (face.xmin, face.xmax),
                    Direction::Y => (face.ymin, face.ymax),
                };

                // no intersection, we overshot the final box
                if min_ > rect_end {
                    break;
                }
                // no intersection
                else if max_ < rect_start {
                    continue;
                }
                // full intersection
                else if min_ >= rect_start && max_ <= rect_end {
                    num_faces += 1.0;
                    faces_area += face.area();
                    continue;
                }

                // partial intersection
                if min_ <= rect_end && max_ > rect_end {
                    num_faces += (rect_end - min_) as f32 / (max_ - min_) as f32;
                    faces_area += (rect_end - min_)
                        * match direction {
                            Direction::X => face.height(),
                            Direction::Y => face.width(),
                        };
                    continue;
                }
            }

            // update max_boxes
            if num_faces > 0.0 {
                if num_faces > max_faces {
                    max_faces = num_faces;
                    faces_info = vec![FaceInfo {
                        area: faces_area,
                        start: rect_start,
                    }];
                } else if (num_faces - max_faces).abs() < f32::EPSILON {
                    faces_info.push(FaceInfo {
                        area: faces_area,
                        start: rect_start,
                    });
                }
            }
        }

        faces_info.sort_by_key(|face_info| (face_info.area, face_info.start));
        // use the match with the maximum area of face coverage
        let max_face_area = faces_info.last().expect("could not get max face area").area;
        faces_info.retain(|face_info| face_info.area == max_face_area);

        self.clamp(
            faces_info[faces_info.len() / 2].start as f32,
            &direction,
            target_width,
            target_height,
        )
    }

    pub fn crop_candidates(&mut self, aspect_ratio: &AspectRatio) -> Vec<Face> {
        let (target_width, target_height, direction) = self.crop_rect(aspect_ratio);

        if self.width == target_width && self.height == target_height {
            return vec![Face {
                xmax: self.width,
                ymax: self.height,
                ..Face::default()
            }];
        }

        if self.faces.len() == 1 {
            vec![self.crop_single_face(&direction, target_width, target_height)]
        } else {
            self.faces.sort_by_key(|face| match direction {
                Direction::X => face.xmin,
                Direction::Y => face.ymin,
            });

            let mut faces_info: Vec<FaceInfo> = vec![];
            let (rect_max, rect_len) = match direction {
                Direction::X => (self.width - target_width, target_width),
                Direction::Y => (self.height - target_height, target_height),
            };

            for rect_start in 0..rect_max {
                // check number of faces in decimal within enclosed within larger rectangle
                let rect_end = rect_start + rect_len;

                for face in &self.faces {
                    let (min_, max_) = match direction {
                        Direction::X => (face.xmin, face.xmax),
                        Direction::Y => (face.ymin, face.ymax),
                    };

                    // no intersection, we overshot the final box
                    if min_ > rect_end {
                        break;
                    }
                    // no intersection
                    else if max_ < rect_start {
                        continue;
                    }
                    // full intersection
                    else if min_ >= rect_start && max_ <= rect_end {
                        faces_info.push(FaceInfo {
                            area: face.area(),
                            start: rect_start,
                        });
                        continue;
                    }
                }
            }

            faces_info.sort_by_key(|face_info| (face_info.area, face_info.start));

            // group faces by area
            let faces_by_area: HashMap<_, Vec<_>> =
                faces_info
                    .iter()
                    .fold(HashMap::new(), |mut acc, face_info| {
                        acc.entry(face_info.area).or_default().push(face_info.start);
                        acc
                    });

            faces_by_area
                .values()
                .map(|faces| {
                    let mid = faces[faces.len() / 2];
                    self.clamp(mid as f32, &direction, target_width, target_height)
                })
                .sorted_by_key(|face| match direction {
                    Direction::X => face.xmin,
                    Direction::Y => face.ymin,
                })
                .collect()
        }
    }
}
