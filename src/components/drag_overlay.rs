#![allow(non_snake_case)]
use dioxus::prelude::*;

use wallfacer::{dragger::Dragger, geometry::Geometry};

use crate::state::Wall;

#[component]
pub fn DragOverlay(wall: Signal<Wall>, geom: Geometry, dragger: Signal<Dragger>) -> Element {
    rsx! {
        div {
            class: "absolute bg-black bg-opacity-60 inset-0",
            style: dragger().overlay_style(&geom),
            onmouseup: move |_| {
                dragger.with_mut(|dragger| {
                    dragger.is_dragging = true;
                });
            },
            onmousemove: {
                move |evt| {
                    if dragger().is_dragging && evt.held_buttons().contains(dioxus::html::input_data::MouseButton::Primary) {
                        let (new_x, new_y) = evt.element_coordinates().into();

                        wall.with_mut(|wall| {
                            let new_geom = dragger().update((new_x, new_y), &geom);
                            wall.set_geometry(&new_geom);
                        });

                        dragger.with_mut(|dragger| {
                            dragger.x = new_x;
                            dragger.y = new_y;
                        });
                    }
                }
            },
        }
    }
}
