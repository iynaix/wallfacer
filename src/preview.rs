#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::Direction,
    geometry::Geometry,
    wallpapers::{Face, WallInfo},
};

use crate::app_state::UiState;

#[component]
fn FacesOverlay(faces: Vec<Face>, image_dimensions: (f64, f64)) -> Element {
    if faces.is_empty() {
        return None;
    }

    let (img_w, img_h) = image_dimensions;

    rsx! {
        {faces.iter().map(|face| {
            let start_x = f64::from(face.xmin) / img_w * 100.0;
            let start_y = f64::from(face.ymin) / img_h * 100.0;

            let w = f64::from(face.xmax - face.xmin) / img_w * 100.0;
            let h = f64::from(face.ymax - face.ymin) / img_h * 100.0;

            rsx! {
                div {
                    class: "absolute border-2 border-red-500",
                    style: format!("top: {start_y}%; left: {start_x}%; width: {w}%; height: {h}%;"),
                }
            }
        })}
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DraggableImageProps {
    image: String,
    image_dimensions: (f64, f64),
    direction: Direction,
    geometry: Geometry,
    final_dimensions: (f64, f64),
    ui: Signal<UiState>,
}

pub fn DraggableImage(mut props: DraggableImageProps) -> Element {
    let mut is_dragging = use_signal(|| false);
    let mut drag_coords = use_signal(|| (0.0, 0.0));
    let cls = match props.direction {
        Direction::X => "cursor-ew-resize",
        Direction::Y => "cursor-ns-resize",
    };

    let dir = props.direction;
    let geom = props.geometry;
    let (img_w, img_h) = props.image_dimensions;
    let (final_w, final_h) = props.final_dimensions;

    rsx! {
        img {
            src: "{props.image}",
            class: cls,
            onmousedown: move |evt| {
                is_dragging.set(true);
                drag_coords.set(evt.client_coordinates().into());
            },
            onmouseup: move |_| {
                is_dragging.set(false);
            },
            onmousemove: {
                move |evt| {
                    if is_dragging() {
                        let (x, y) = drag_coords();
                        let (new_x, new_y) = evt.client_coordinates().into();
                        let (dx, dy) = (new_x - x, new_y - y);

                        let new_geom = match dir {
                            Direction::X => {
                                let scaled_dx = img_w / final_w * dx;
                                Geometry {
                                    x: (f64::from(geom.x) + scaled_dx).clamp(0.0, img_w - f64::from(geom.w)) as u32,
                                    ..geom.clone()
                                }
                            },
                            Direction::Y => {
                                let scaled_dy = img_h / final_h * dy;
                                Geometry {
                                    y: (f64::from(geom.y) + scaled_dy).clamp(0.0, img_h - f64::from(geom.h)) as u32,
                                    ..geom.clone()
                                }
                            },
                        };

                        props.ui.with_mut(|ui| {
                            ui.preview_geometry = Some(new_geom);
                        });
                        drag_coords.set((new_x, new_y));
                    }
                }
            },
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct PreviewerProps {
    wall_info: WallInfo,
    ui: Signal<UiState>,
}

pub fn Previewer(props: PreviewerProps) -> Element {
    // store the final rendered width and height of the image
    let mut final_dimensions = use_signal(|| (0.0, 0.0));
    let info = props.wall_info;
    let ui = (props.ui)();

    let path = info.path();
    let path = path
        .to_str()
        .expect("could not convert path to str")
        .to_string();

    // preview geometry takes precedence
    let geom = match ui.preview_geometry {
        Some(g) => g,
        None => info.get_geometry(&ui.ratio),
    };

    let (dir, start_ratio, end_ratio) = info.overlay_transforms(&geom);

    let (img_w, img_h) = info.image_dimensions_f64();
    let start_cls = match dir {
        Direction::X => "origin-left top-0 left-0",
        Direction::Y => "origin-top top-0 left-0",
    };

    let end_cls = match dir {
        Direction::X => "origin-right top-0 right-0",
        Direction::Y => "origin-bottom bottom-0 left-0",
    };

    let overlay_cls = format!(
        "absolute bg-black bg-opacity-60 w-full h-full transition-transform ease-in-out {}",
        // don't apply transitions in slider mode
        if ui.manual_mode { "" } else { "transition" }
    );

    rsx! {
        div {
            class: "relative",
            // store the final rendered width and height of the image
            onmounted: move |evt| {
                async move {
                    let coords = evt.get_client_rect().await.expect("could not get client rect");
                    let elem_width = coords.width();
                    final_dimensions.set((elem_width, (elem_width / img_w * img_h).floor()));
                }
            },
            if ui.manual_mode {
                DraggableImage {
                    image: path,
                    image_dimensions: (img_w, img_h),
                    direction: dir.clone(),
                    geometry: geom,
                    final_dimensions: final_dimensions(),
                    ui: props.ui,
                }
            } else {
                img { src: "{path}" }
            }
            div {
                class: "{overlay_cls} {start_cls}",
                style: format!("transform: scale{}({})", dir, start_ratio),
            }
            div {
                class: "{overlay_cls} {end_cls}",
                style: format!("transform: scale{}({})", dir, end_ratio),
            }

            if ui.show_faces {
                FacesOverlay {
                    faces: info.faces,
                    image_dimensions: (img_w, img_h),
                }
            }
        }
    }
}
