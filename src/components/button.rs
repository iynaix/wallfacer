#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallfacer::geometry::Geometry;

use crate::state::Wall;

#[component]
pub fn Button(
    class: Option<String>,
    title: Option<String>,
    active: Option<bool>,
    spin: Option<bool>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmouseenter: Option<EventHandler<MouseEvent>>,
    onmouseleave: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let active_cls = if active.unwrap_or_default() {
        "bg-active"
    } else {
        "bg-surface0"
    };

    rsx! {
        button {
            r#type: "button",
            class: format!("relative inline-flex items-center px-3 py-2 font-semibold text-text ring-1 ring-inset ring-surface1 hover:bg-crust focus:z-10 {active_cls} {}", class.unwrap_or_default()),
            title,
            disabled: spin.unwrap_or_default(),
            onclick: move |evt| {
                if !spin.unwrap_or_default() {
                    if let Some(handler) = &onclick {
                        handler.call(evt);
                    }
                }
            },
            onmouseenter: move |evt| {
                if !spin.unwrap_or_default() {
                    if let Some(handler) = &onmouseenter {
                        handler.call(evt);
                    }
                }
            },
            onmouseleave: move |evt| {
                if !spin.unwrap_or_default() {
                    if let Some(handler) = &onmouseleave {
                        handler.call(evt);
                    }
                }
            },
            if spin.unwrap_or_default() {
                svg {
                    "fill": "none",
                    role: "status",
                    "xmlns": "http://www.w3.org/2000/svg",
                    "viewBox": "0 0 100 101",
                    // "aria-hidden": "true",
                    class: "inline w-4 h-4 me-3 text-white animate-spin",
                    path {
                        "d": "M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z",
                        "fill": "#E5E7EB"
                    }
                    path {
                        "d": "M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z",
                        "fill": "currentColor"
                    }
                }
            }
            {children},
        }
    }
}

#[component]
pub fn PreviewableButton(
    class: Option<String>,
    active: Option<bool>,
    geom: Geometry,
    title: Option<String>,
    wall: Signal<Wall>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmouseenter: Option<EventHandler<MouseEvent>>,
    onmouseleave: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    const PREVIEW_DELAY: u64 = 300;
    // only show preview after the user has hovered over the button for a short period
    let mut is_hovering = use_signal(|| false);

    use_effect(move || {
        if !is_hovering() {
            wall.with_mut(|wall| {
                wall.mouseover_geom = None;
            });
        }
    });

    rsx! {
        Button {
            class,
            active,
            title,
            onclick: move |evt| {
                is_hovering.set(false);

                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            onmouseenter: move |_| {
                let geom = geom.clone();
                is_hovering.set(true);

                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(PREVIEW_DELAY)).await;
                    // only proceed if user is still hovering over the button
                    if is_hovering() {
                        wall.with_mut(|wall| {
                            wall.mouseover_geom = Some(geom);
                        });
                    }
                });
            },
            onmouseleave: move |_| {
                is_hovering.set(false);
            },
            {children}
        }
    }
}
