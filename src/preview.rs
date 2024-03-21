#![allow(non_snake_case)]
use dioxus::prelude::*;
use wallpaper_ui::{
    cropper::{AspectRatio, Direction},
    wallpapers::WallInfo,
};

#[derive(Clone, PartialEq, Props)]
pub struct PreviewerProps {
    wall_info: Signal<WallInfo>,
    ratio: AspectRatio,
    #[props(default = false)]
    show_faces: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub fn Previewer(props: PreviewerProps) -> Element {
    let info = (props.wall_info)();
    let path = info.path();
    let path = path.to_str().expect("could not convert path to str");

    let geometry = info.get_geometry(&props.ratio);

    let (dir, start_ratio, end_ratio) = info.overlay_transforms(&geometry);

    let (img_w, img_h) = info.image_dimensions();
    let faces: Vec<_> = if props.show_faces {
        info.faces.iter().map(|face| {
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

    let start_cls = match dir {
        Direction::X => "origin-left top-0 left-0",
        Direction::Y => "origin-top top-0 left-0",
    };
    let start_style = match dir {
        Direction::X => format!("transform: scaleX({})", start_ratio),
        Direction::Y => format!("transform: scaleY({})", start_ratio),
    };

    let end_cls = match dir {
        Direction::X => "origin-right top-0 right-0",
        Direction::Y => "origin-bottom bottom-0 left-0",
    };
    let end_style = match dir {
        Direction::X => format!("transform: scaleX({})", end_ratio),
        Direction::Y => format!("transform: scaleY({})", end_ratio),
    };

    let overlay_cls =
        "absolute bg-black bg-opacity-60 w-full h-full transition transition-transform ease-in-out";

    rsx! {
        div {
            class: "mx-4 mt-4 mb-16",
            position: "relative",
            img {
                src: "{path}",
            }
            div {
                class: "{overlay_cls} {start_cls}",
                style: start_style,
            }
            div {
                class: "{overlay_cls} {end_cls}",
                style: end_style,
            }
            for face in faces {
                {face}
            }
        }
    }
}
