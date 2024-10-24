#![allow(non_snake_case)]
use crate::{
    components::{drag_overlay::DragOverlay, use_ui},
    state::Wall,
};
use dioxus::prelude::*;
use dioxus_sdk::utils::window::{use_window_size, WindowSize};
use wallfacer::{cropper::Direction, dragger::Dragger, wallpapers::WallInfo};

#[component]
fn FacesOverlay(info: WallInfo) -> Element {
    if info.faces.is_empty() {
        return None;
    }

    let (img_w, img_h) = info.dimensions_f64();

    rsx! {
        {info.faces.iter().map(|face| {
            let start_x = f64::from(face.x) / img_w * 100.0;
            let start_y = f64::from(face.y) / img_h * 100.0;

            let w = f64::from(face.w) / img_w * 100.0;
            let h = f64::from(face.h) / img_h * 100.0;

            rsx! {
                div {
                    class: "absolute border-2 border-red-500",
                    // pointer-events: none to allow mouse events to pass through
                    style: "top: {start_y}%; left: {start_x}%; width: {w}%; height: {h}%; pointer-events: none;",
                }
            }
        })}
    }
}

/// fit the image within the max preview area
fn get_preview_size(min_y: f64, win_size: WindowSize, img: (f64, f64)) -> (f64, f64) {
    let scale = dioxus::desktop::window().scale_factor();
    let margin: f64 = 16.0 * scale;
    let candidate_btns: f64 = 36.0 * scale;
    let min_y = min_y * scale;

    let max_w = margin.mul_add(-2.0, f64::from(win_size.width));
    // handle extra space for candidate buttons
    let max_h = f64::from(win_size.height) - min_y - margin - (candidate_btns + margin);

    let (img_w, img_h) = img;

    let mut final_w = max_w;
    let mut final_h = max_w / img_w * img_h;

    if final_h > max_h {
        final_h = max_h;
        final_w = max_h / img_h * img_w;
    }

    // cannot be larger than the image
    if final_w > img_w || final_h > img_h {
        (img_w / scale, img_h / scale)
    } else {
        (final_w / scale, final_h / scale)
    }
}

#[component]
pub fn Previewer(wall: Signal<Wall>) -> Element {
    let ui = use_ui();

    // store y coordinate of the previewer
    let mut preview_y = use_signal(|| 0.0);
    let window_size = use_window_size();
    // calculate the preview size of the image
    // only needs to change when the window resizes
    let mut dragger = use_signal(|| {
        let (image_w, image_h) = wall().current.dimensions_f64();
        let wh = get_preview_size(preview_y(), window_size(), (image_w, image_h));
        Dragger::new((image_w, image_h), wh)
    });

    // update dragger on resize
    use_effect(move || {
        // use wall() to initiate a refresh
        let (image_w, image_h) = wall().current.dimensions_f64();
        let (w, h) = get_preview_size(preview_y(), window_size(), wall().current.dimensions_f64());

        dragger.with_mut(|dragger| {
            dragger.image_w = image_w;
            dragger.image_h = image_h;
            dragger.preview_w = w;
            dragger.preview_h = h;
        });
    });

    let ui = ui();

    // preview geometry takes precedence
    let geom = wall()
        .mouseover_geom
        .unwrap_or_else(|| wall().get_geometry());

    rsx! {
        div {
            class: "relative m-auto",
            style: "width: {dragger().preview_w}px; height: {dragger().preview_h}px;",
            img {
                class: match dragger().direction(&geom) {
                    Direction::X => "cursor-ew-resize",
                    Direction::Y => "cursor-ns-resize",
                },
                src: wall().path(),
                // store the final rendered width and height of the image
                onmounted: move |evt| {
                    async move {
                        let coords = evt.get_client_rect().await.expect("could not get client rect");
                        // store the y coordinate of the previewer, the rest can be calculated from there
                        preview_y.set(coords.min_y());
                    }
                },
                // clip-path produces a "hole", so detect click events on the image
                onmousedown: move |evt| {
                    let (x, y) = evt.element_coordinates().into();
                    dragger.with_mut(|dragger| {
                        dragger.is_dragging = true;
                        dragger.x = x;
                        dragger.y = y;
                    });
                },
                onmousemove: move |evt| {
                    if dragger().is_dragging && evt.held_buttons().contains(dioxus::html::input_data::MouseButton::Primary) {
                        let (new_x, new_y) = evt.element_coordinates().into();

                        wall.with_mut(|wallpapers| {
                            let new_geom = dragger().update((new_x, new_y), &geom);
                            wallpapers.set_geometry(&new_geom);
                        });

                        dragger.with_mut(|dragger| {
                            dragger.x = new_x;
                            dragger.y = new_y;
                        });
                    }
                },
                onmouseup: move |_| {
                    dragger.with_mut(|dragger| {
                        dragger.is_dragging = true;
                    });
                },
            }

            if ui.show_faces {
                FacesOverlay { info: wall().current }
            }

            DragOverlay {
                wall,
                geom: geom.clone(),
                dragger,
            }
        }
    }
}
