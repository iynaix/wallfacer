#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    buttons::Button,
    dropdown::{Dropdown, DropdownOptions},
};

use wallust::{
    backends::Backend,
    colorspaces::{ColorSpace, FallbackGenerator},
    palettes::Palette,
};

fn wallust_config() -> wallust::config::Config {
    wallust::config::Config::new(
        &dirs::config_dir().unwrap_or_else(|| panic!("could not get XDG config directory")),
        None,
        None,
        false,
    )
    .unwrap_or_else(|_| panic!("unable to read wallust.toml"))
}

#[component]
pub fn Wallust() -> Element {
    let mut conf = use_signal(wallust_config);

    let backend = DropdownOptions::new(vec![
        Backend::Full,
        Backend::Resized,
        Backend::Wal,
        Backend::Thumb,
        Backend::FastResize,
        Backend::Kmeans,
    ]);

    let colorspace = DropdownOptions::new(vec![
        ColorSpace::Lab,
        ColorSpace::LabMixed,
        ColorSpace::Lch,
        ColorSpace::LchMixed,
    ]);

    let fallback_generator = DropdownOptions::new(vec![
        FallbackGenerator::Interpolate,
        FallbackGenerator::Complementary,
    ]);

    let palettes = DropdownOptions::new(vec![
        Palette::Dark16,
        Palette::DarkComp16,
        Palette::HardDark16,
        Palette::HardDarkComp16,
        Palette::SoftDark16,
        Palette::SoftDarkComp16,
    ])
    .to_label(|v| v.to_string().replace("16", ""));

    // TODO: saturation and threshold

    rsx! {
        div {
            class: "flex flex-col gap-y-4",
            Dropdown {
                name: "Palette",
                options: palettes,
                value: conf.read().palette,
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.palette = new_value;
                    });
                }
            }

            Dropdown {
                name: "Backend",
                options: backend,
                value: conf.read().backend,
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.backend = new_value;
                    });
                }
            }

            Dropdown {
                name: "Colorspace",
                options: colorspace,
                value: conf.read().color_space,
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.color_space = new_value;
                    });
                }
            }

            Dropdown {
                name: "Fallback Generator",
                options: fallback_generator,
                value: conf.read().fallback_generator.unwrap_or_default(),
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.fallback_generator = Some(new_value);
                    });
                }
            }

            Button {
                onclick: move |_| {
                },
                "Reset"
            }

            Button {
                onclick: move |_| {
                },
                "Preview"
            }
        }
    }
}
