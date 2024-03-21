#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::{AspectRatio, Direction},
    wallpapers::WallInfo,
};

#[derive(Clone, PartialEq, Eq, Props)]
pub struct PreviewerProps {
    wall_info: WallInfo,
    ratio: AspectRatio,
    #[props(default = false)]
    show_faces: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub fn Previewer(props: PreviewerProps) -> Element {
    let path = props.wall_info.path();
    let path = path.to_str().expect("could not convert path to str");

    let geometry = props.wall_info.get_geometry(&props.ratio);

    let (dir, start_pct, end_pct) = props.wall_info.overlay_percents(&geometry);

    let start_cls = match dir {
        Direction::X => "top-0 left-0 h-full",
        Direction::Y => "top-0 left-0 w-full",
    };
    let start_style = match dir {
        Direction::X => format!("width: {start_pct}%"),
        Direction::Y => format!("height: {start_pct}%"),
    };

    let end_cls = match dir {
        Direction::X => "top-0 right-0 h-full",
        Direction::Y => "bottom-0 left-0 w-full",
    };
    let end_style = match dir {
        Direction::X => format!("width: {end_pct}%"),
        Direction::Y => format!("height: {end_pct}%"),
    };

    let (img_w, img_h) = props.wall_info.image_dimensions();
    let faces: Vec<_> = if props.show_faces {
        props.wall_info.faces.iter().map(|face| {
            let start_x = face.xmin as f32 / img_w as f32 * 100.0;
            let start_y = face.ymin as f32 / img_h as f32 * 100.0;

            let w = face.width() as f32 / img_w as f32 * 100.0;
            let h = face.height() as f32 / img_h as f32 * 100.0;

            rsx! {
                div {
                    class: "absolute border-2 border-red-500",
                    style: format!("top: {start_y}%; left: {start_x}%; width: {w}%; height: {h}%;"),
                }
            }
        }).collect()
    } else {
        Vec::new()
    };

    rsx! {
        div {
            class: "mx-4 mt-4 mb-16",
            position: "relative",
            img {
                src: "{path}",
            }
            div {
                class: "absolute bg-black bg-opacity-60 {start_cls}",
                style: start_style,
            }
            div {
                class: "absolute bg-black bg-opacity-60 {end_cls}",
                style: end_style,
            }
            for face in faces {
                {face}
            }
        }
    }
}
