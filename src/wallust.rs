#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::{
    buttons::Button,
    dropdown::{Dropdown, DropdownOptions},
    slider::Slider,
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

    rsx! {
        div {
            class: "flex flex-wrap w-full gap-y-8",
            Dropdown {
                name: "Palette",
                class: "w-1/2 py-4 px-8",
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
                class: "w-1/2 py-4 px-8",
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
                class: "w-1/2 py-4 px-8",
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
                class: "w-1/2 py-4 px-8",
                options: fallback_generator,
                value: conf.read().fallback_generator.unwrap_or_default(),
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.fallback_generator = Some(new_value);
                    });
                }
            }

            Slider {
                name: "Saturation",
                class: "w-1/2 py-4 px-8",
                value: conf.read().saturation.unwrap_or_default(),
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.saturation = if new_value == 0 {
                            None
                        } else {
                            Some(new_value)
                        };
                    });
                }
            }

            Slider {
                name: "Threshold",
                class: "w-1/2 py-4 px-8",
                value: conf.read().threshold.unwrap_or_default(),
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.threshold = if new_value == 0 {
                            None
                        } else {
                            Some(new_value)
                        };
                    });
                }
            }

            div {
                class: "w-full flex justify-center gap-x-8",
                Button {
                    onclick: move |_| {
                        conf.set(wallust_config());
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
}
