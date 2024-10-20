#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::components::use_wallpapers;
use wallfacer::{dragger::Dragger, geometry::Geometry};

#[component]
pub fn DragOverlay(geometry: Geometry, dragger: Signal<Dragger>) -> Element {
    let mut wallpapers = use_wallpapers();

    rsx! {
        div {
            class: "absolute bg-black bg-opacity-60 inset-0",
            style: dragger().overlay_style(&geometry),
            onmouseup: move |_| {
                dragger.with_mut(|dragger| {
                    dragger.is_dragging = true;
                });
            },
            onmousemove: {
                move |evt| {
                    if dragger().is_dragging && evt.held_buttons().contains(dioxus::html::input_data::MouseButton::Primary) {
                        let (new_x, new_y) = evt.element_coordinates().into();

                        wallpapers.with_mut(|wallpapers| {
                            let new_geom = dragger().update((new_x, new_y), &geometry);
                            wallpapers.set_geometry(&new_geom);
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
