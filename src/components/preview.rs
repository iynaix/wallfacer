#![allow(non_snake_case)]

use crate::{components::use_ui, state::Wall};
use dioxus::{
    desktop::{
        LogicalPosition,
        muda::{self, ContextMenu},
        tao::platform::unix::WindowExtUnix,
        wry::dpi::Position,
    },
    prelude::*,
};
use wallfacer::{cropper::Direction, geometry::Geometry};

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

fn show_context_menu(x: f64, y: f64, face: &Geometry) {
    let menu = muda::Submenu::with_items(
        "Face menu",
        true,
        &[&muda::MenuItem::with_id(
            format!("center-face|{face}"),
            "Center on Face",
            true,
            None,
        )],
    )
    .expect("unable to create context menu");

    let window = dioxus::desktop::window();
    let gtk_window = window.window.gtk_window();

    menu.show_context_menu_for_gtk_window(
        gtk_window.as_ref(),
        Some(Position::Logical(LogicalPosition { x, y })),
    );
}

#[component]
fn FacesOverlay(wall: Signal<Wall>, direction: Direction) -> Element {
    dioxus::desktop::use_muda_event_handler(move |evt| {
        if let Some((id, face)) = evt.id().as_ref().split_once('|') {
            match id {
                "center-face" => {
                    if let Some(face) = wall().current.faces.iter().find(|f| f.to_string() == face)
                    {
                        wall.with_mut(|wallpaper| {
                            wallpaper.set_geometry(&wallpaper.center_on_face(face));
                        });
                    }
                }
                _ => {}
            }
        }
    });

    let info = wall().current;
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
    let mut elem_wh = use_signal(|| (0.0, 0.0));

    let ui = use_ui();

    // preview geometry takes precedence
    let geom = wall()
        .mouseover_geom
        .unwrap_or_else(|| wall().get_geometry());

    let (img_w, img_h) = wall().current.dimensions_f64();

    // get direction of the geometry
    let direction = if wall().current.height == geom.h {
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
                    onresize: move |evt| {
                        if let Ok(size) = evt.data.get_content_box_size() {
                            elem_wh.set((size.width, size.height));
                        }
                    },
                    // the overlays produce a "hole", so detect click events there
                    onmousedown: move |evt| {
                        async move {
                            let (x, y) = evt.element_coordinates().into();

                            is_dragging.set(true);
                            dragger.set((x, y));
                        }
                    },
                    oncontextmenu: move |evt| {
                        let (x, y) = evt.element_coordinates().into();
                        let (elem_w, elem_h) = elem_wh();

                        // normalize to absolute image coordinates
                        let img_x = x / elem_w * img_w;
                        let img_y = y / elem_h * img_h;

                        if let Some(face) = wall().current.faces.iter().find(|face|
                            face.contains(img_x as u32, img_y as u32)
                        ) {
                            let (x, y) = evt.client_coordinates().into();
                            show_context_menu(x, y, face);
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

                            wall.with_mut(|wallpaper| {
                                wallpaper.set_geometry(&new_geom);
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
                    FacesOverlay { wall, direction }
                }
            }
        }
    }
}
