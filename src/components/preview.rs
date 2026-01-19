#![allow(non_snake_case)]

use crate::{components::use_ui, state::Wall};
use dioxus::prelude::*;
use wallfacer::{cropper::Direction, geometry::Geometry, wallpapers::WallInfo};

fn get_overlay_styles(
    img_w: f64,
    img_h: f64,
    direction: Direction,
    geom: &Geometry,
) -> (String, String) {
    match direction {
        Direction::X => (
            format!(
                "transform-origin: left; transform: scaleX({});",
                f64::from(geom.x) / img_w
            ),
            format!(
                "transform-origin: right; transform: scaleX({});",
                (img_w - f64::from(geom.x + geom.w)) / img_w,
            ),
        ),
        Direction::Y => (
            format!(
                "transform-origin: top; transform: scaleY({});",
                f64::from(geom.y) / img_h,
            ),
            format!(
                "transform-origin: bottom; transform: scaleY({});",
                (img_h - f64::from(geom.y + geom.h)) / img_h,
            ),
        ),
    }
}

#[component]
fn FacesOverlay(info: WallInfo) -> Element {
    if info.faces.is_empty() {
        return rsx! {};
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
                    // pointer-events: none to allow mouse events to pass through
                    class: "absolute border-2 bg-transparent border-red-500 inset-0 pointer-events-none transform-gpu origin-top-left",
                    style: format!("width: {w}%; height: {h}%; top: {start_y}%; left: {start_x}%;"),
                }
            }
        })}
    }
}

#[component]
pub fn Previewer(wall: Signal<Wall>) -> Element {
    let mut is_dragging = use_signal(|| false);
    let mut dragger = use_signal::<(f64, f64)>(|| (0.0, 0.0));
    let mut img_elem = use_signal(|| None);
    let mut elem_wh = use_signal(|| (0.0, 0.0));

    let ui = use_ui();

    // preview geometry takes precedence
    let geom = wall()
        .mouseover_geom
        .unwrap_or_else(|| wall().get_geometry());

    let (img_w, img_h) = wall().current.dimensions_f64();

    // get direction of the geometry
    let direction = if (img_h - f64::from(geom.h)).abs() < f64::EPSILON {
        Direction::X
    } else {
        Direction::Y
    };

    let cursor_cls = match direction {
        Direction::X => "cursor-ew-resize",
        Direction::Y => "cursor-ns-resize",
    };

    let pointer_cls = if is_dragging() {
        "pointer-events-none"
    } else {
        ""
    };

    let (start_overlay_style, end_overlay_style) =
        get_overlay_styles(img_w, img_h, direction, &geom);

    let overlay_cls =
        "absolute bg-black/60 inset-0 transform-gpu isolate transition will-change-transform";

    rsx! {
        div {
            class: "flex items-center justify-center min-h-0 min-w-0 px-4 pb-4 {cursor_cls}",

            div {
                class: "relative m-auto max-h-full max-w-full",
                style: "aspect-ratio: {wall().current.width} / {wall().current.height};",

                img {
                    src: wall().path(),
                    class: "w-full h-full object-contain object-center block",
                    onmounted: move |evt| {
                        // coordinates are requested when clicked since it is initially zero size
                        img_elem.set(Some(evt.data()));
                    },
                    // the overlays produce a "hole", so detect click events there
                    onmousedown: move |evt| {
                        async move {
                            let (x, y) = evt.element_coordinates().into();

                            is_dragging.set(true);
                            dragger.set((x, y));

                            if let Some(elem) = img_elem() {
                                let rect = elem.get_client_rect().await.expect("could not get client rect");
                                elem_wh.set((rect.width(), rect.height()));
                            }
                        }
                    },
                    onmouseup: move |_| {
                        is_dragging.set(false);
                    },
                    onmousemove: move |evt| {
                        if is_dragging() && evt.held_buttons().contains(dioxus::html::input_data::MouseButton::Primary) {
                            let (new_x, new_y) = evt.element_coordinates().into();
                            let (x, y) = dragger();
                            let (elem_w, elem_h) = elem_wh();

                            let new_geom = match direction {
                                Direction::X => {
                                    let dx = img_w / elem_w * (new_x - x);
                                    Geometry {
                                        x: (f64::from(geom.x) + dx).clamp(0.0, img_w - f64::from(geom.w)) as u32,
                                        ..geom.clone()
                                    }
                                }
                                Direction::Y => {
                                    let dy = img_h / elem_h * (new_y - y);
                                    Geometry {
                                        y: (f64::from(geom.y) + dy).clamp(0.0, img_h - f64::from(geom.h)) as u32,
                                        ..geom.clone()
                                    }
                                }
                            };

                            wall.with_mut(|wallpapers| {
                                wallpapers.set_geometry(&new_geom);
                            });

                            dragger.set((new_x, new_y));
                        }
                    },
                }

                // start overlay
                div {
                    class: "{overlay_cls} {pointer_cls}",
                    style: start_overlay_style,
                }

                // end overlay
                div {
                    class: "{overlay_cls} {pointer_cls}",
                    style: end_overlay_style,
                }

                if ui().show_faces {
                    FacesOverlay { info: wall().current }
                }
            }
        }
    }
}
