#![allow(non_snake_case)]
use dioxus::prelude::*;

use wallpaper_ui::{cropper::Direction, geometry::Geometry};

use crate::app_state::Wallpapers;

#[component]
pub fn DraggableOverlay(
    dimensions: (f64, f64),
    image_dimensions: (f64, f64),
    overlay_ratios: (f64, f64),
    direction: Direction,
    geometry: Geometry,
    wallpapers: Signal<Wallpapers>,
) -> Element {
    let mut is_dragging = use_signal(|| false);
    let mut drag_coords = use_signal(|| (0.0, 0.0));

    let (img_w, img_h) = image_dimensions;
    let (final_w, final_h) = dimensions;

    rsx! {
        div {
            class: "absolute w-full origin-top-left top-0 left-0",
            class: match &direction {
                Direction::X => "cursor-ew-resize",
                Direction::Y => "cursor-ns-resize",
            },
            style: "height: {final_h}px",
            onmousedown: {
                move |evt| {
                    // only initiate drag if mouse is in clear zone
                    let (x, y) = evt.element_coordinates().into();
                    let in_clear_zone = match direction {
                        Direction::X => {
                            // NOTE: p-4 on left side
                            let pct = x / final_w;
                            overlay_ratios.0 < pct && pct < overlay_ratios.1
                        },
                        Direction::Y => {
                            let pct = y / final_h;
                            overlay_ratios.0 < pct || pct < overlay_ratios.1
                        },
                    };

                    if in_clear_zone {
                        is_dragging.set(true);
                        drag_coords.set((x, y));
                    }
                }

            },
            onmouseup: move |_| {
                is_dragging.set(false);
            },
            onmousemove: {
                move |evt| {
                    if is_dragging() && evt.held_buttons().contains(dioxus::html::input_data::MouseButton::Primary) {
                        let (x, y) = drag_coords();
                        let (new_x, new_y) = evt.element_coordinates().into();
                        let (dx, dy) = (new_x - x, new_y - y);

                        let new_geom = match direction {
                            Direction::X => {
                                let scaled_dx = img_w / final_w * dx;
                                Geometry {
                                    x: (f64::from(geometry.x) + scaled_dx).clamp(0.0, img_w - f64::from(geometry.w)) as u32,
                                    ..geometry.clone()
                                }
                            },
                            Direction::Y => {
                                let scaled_dy = img_h / final_h * dy;
                                Geometry {
                                    y: (f64::from(geometry.y) + scaled_dy).clamp(0.0, img_h - f64::from(geometry.h)) as u32,
                                    ..geometry.clone()
                                }
                            },
                        };
                        wallpapers.with_mut(|wallpapers| {
                            wallpapers.set_geometry(&new_geom);
                        });
                        drag_coords.set((new_x, new_y));
                    }
                }
            },
        }
    }
}
