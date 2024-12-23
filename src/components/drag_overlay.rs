#![allow(non_snake_case)]
use dioxus::prelude::*;

use wallfacer::{dragger::Dragger, geometry::Geometry};

use crate::state::Wall;

#[component]
pub fn DragOverlay(wall: Signal<Wall>, geom: Geometry, dragger: Signal<Dragger>) -> Element {
    let dragger = dragger();
    let (start_style, end_style) = dragger.overlay_styles(&geom);
    let pointer_cls = if dragger.is_dragging {
        "pointer-events-none"
    } else {
        ""
    };

    let overlay_cls =
        "absolute bg-black bg-opacity-60 inset-0 transform-gpu isolate transition will-change-transform";

    rsx! {
        div {
            class: "{overlay_cls} {pointer_cls}",
            style: start_style,
        }
        div {
            class: "{overlay_cls} {pointer_cls}",
            style: end_style,
        }
    }
}
