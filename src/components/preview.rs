#![allow(non_snake_case)]
use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_sdk::utils::window::{use_window_size, WindowSize};
use wallpaper_ui::{cropper::Direction, wallpapers::Face};

use crate::{
    app_state::{PreviewMode, UiState, Wallpapers},
    components::drag_overlay::DragOverlay,
};

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

/// fit the image within the max preview area
fn get_preview_size(
    min_y: f64,
    win_size: WindowSize,
    img: (f64, f64),
    has_candidates: bool,
) -> (f64, f64) {
    let margin: f64 = 16.0;
    let candidate_btns: f64 = 36.0;

    let max_w = margin.mul_add(-2.0, f64::from(win_size.width));
    // handle extra space for candidate buttons
    let reserved_height = if has_candidates {
        candidate_btns + margin
    } else {
        0.0
    };
    let max_h = f64::from(win_size.height) - min_y - margin - reserved_height;

    let (img_w, img_h) = img;

    let mut final_w = max_w;
    let mut final_h = max_w / img_w * img_h;

    if final_h > max_h {
        final_h = max_h;
        final_w = max_h / img_h * img_w;
    }

    (final_w, final_h)
}

#[component]
pub fn Previewer(
    wallpapers: Signal<Wallpapers>,
    ui: Signal<UiState>,
    wallpapers_path: PathBuf,
) -> Element {
    // store y coordinate of the previewer
    let mut preview_y = use_signal(|| 0.0);
    let info = wallpapers().current;
    let ui = ui();

    let path = wallpapers_path.join(&info.filename);
    let path = path
        .to_str()
        .unwrap_or_else(|| panic!("could not convert {path:?} to str"))
        .to_string();

    let is_manual = matches!(ui.preview_mode, PreviewMode::Pan);
    let overlay_cls = "absolute bg-black bg-opacity-60 w-full h-full";

    // preview geometry takes precedence
    let geom = if let PreviewMode::Candidate(Some(mouseover_geom)) = ui.preview_mode {
        mouseover_geom
    } else {
        wallpapers().get_geometry()
    };

    let (direction, start_ratio, end_ratio) = info.overlay_transforms(&geom);

    let img_w = f64::from(info.width);
    let img_h = f64::from(info.height);
    let start_cls = match direction {
        Direction::X => "origin-left top-0 left-0",
        Direction::Y => "origin-top top-0 left-0",
    };

    let end_cls = match direction {
        Direction::X => "origin-right top-0 right-0",
        Direction::Y => "origin-bottom bottom-0 left-0",
    };

    // get preview size of the image
    let (preview_w, preview_h) = get_preview_size(
        preview_y(),
        use_window_size()(),
        (img_w, img_h),
        wallpapers().has_candidates(),
    );

    rsx! {
        div {
            class: "relative m-auto",
            style: "width: {preview_w}px; height: {preview_h}px;",
            img {
                src: path,
                // store the final rendered width and height of the image
                onmounted: move |evt| {
                    async move {
                        let coords = evt.get_client_rect().await.expect("could not get client rect");
                        // store the y coordinate of the previewer, the rest can be calculated from there
                        preview_y.set(coords.min_y());
                    }
                },
            }
            div {
                class: overlay_cls,
                class: start_cls,
                // don't apply transitions in manual mode
                class: if !is_manual { "transition transition-transform ease-linear" },
                style: format!("transform: scale{}({})", direction, start_ratio),
            }
            div {
                class: overlay_cls,
                class: end_cls,
                // don't apply transitions in manual mode
                class: if !is_manual { "transition" },
                style: format!("transform: scale{}({})", direction, end_ratio),
            }

            if is_manual {
                DragOverlay {
                    dimensions: (preview_w, preview_h),
                    image_dimensions: (img_w, img_h),
                    overlay_ratios: (start_ratio, 1.0 - end_ratio),
                    direction,
                    geometry: geom,
                    wallpapers,
                }
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
