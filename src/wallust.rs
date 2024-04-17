#![allow(non_snake_case)]
use clap::Parser;
use dioxus::prelude::*;
use std::{path::Path, process::Command};

use crate::{
    app_state::Wallpapers,
    buttons::Button,
    dropdown::{Dropdown, DropdownOptions},
    slider::Slider,
};

use wallust::{
    backends::Backend,
    colorspaces::{ColorSpace, FallbackGenerator},
    palettes::Palette,
};

#[derive(Debug, Clone, PartialEq)]
struct WallustConfig {
    palette: Palette,
    backend: Backend,
    colorspace: ColorSpace,
    fallback_generator: FallbackGenerator,
    saturation: Option<u8>,
    threshold: Option<u8>,
}

impl From<wallust::config::Config> for WallustConfig {
    fn from(config: wallust::config::Config) -> Self {
        Self {
            palette: config.palette,
            backend: config.backend,
            colorspace: config.color_space,
            fallback_generator: config.fallback_generator.unwrap_or_default(),
            saturation: config.saturation,
            threshold: config.threshold,
        }
    }
}

impl WallustConfig {
    fn load_default() -> wallust::config::Config {
        wallust::config::Config::new(
            &dirs::config_dir().unwrap_or_else(|| panic!("could not get XDG config directory")),
            None,
            None,
            false,
        )
        .unwrap_or_else(|_| panic!("unable to read wallust.toml"))
    }

    fn from_args_str(arg_str: &str) -> Self {
        let cli = wallust::args::WallustArgs::parse_from(
            // add the rest of the command line so it can be parsed by clap
            format!("run {arg_str} a.jpg").split_whitespace(),
        );
        let mut base = Self::load_default();
        base.customs_cli(&cli);
        base.into()
    }

    fn to_args_str(&self) -> String {
        // compare against the base config
        let base_config = Self::load_default();
        let mut new_args = String::new();

        if self.palette != base_config.palette {
            new_args.push_str(&format!(" --palette {}", self.palette));
        }

        if self.backend != base_config.backend {
            new_args.push_str(&format!(" --backend {}", self.backend));
        }

        if self.colorspace != base_config.color_space {
            new_args.push_str(&format!(" --colorspace {}", self.colorspace));
        }

        if self.fallback_generator != base_config.fallback_generator.unwrap_or_default() {
            new_args.push_str(&format!(
                " --fallback-generator {}",
                self.fallback_generator
            ));
        }

        if self.saturation != base_config.saturation {
            new_args.push_str(&format!(
                " --saturation {}",
                self.saturation.unwrap_or_default()
            ));
        }

        if self.threshold != base_config.threshold {
            new_args.push_str(&format!(
                " --threshold {}",
                self.threshold.unwrap_or_default()
            ));
        }

        new_args.trim().to_lowercase()
    }

    fn preview(&self, img: &Path) {
        Command::new("wallust")
            .arg("run")
            .args([
                "--quiet",
                "--no-cache",
                "--check-contrast",
                "--skip-templates",
            ])
            .args(self.to_args_str().split_whitespace())
            .arg(img)
            .spawn()
            .unwrap_or_else(|_| panic!("could not spawn wallust"));
    }
}

#[component]
pub fn Wallust(wallpapers: Signal<Wallpapers>) -> Element {
    let mut conf = use_signal(|| WallustConfig::from_args_str(&wallpapers.read().source.wallust));

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
            class: "flex flex-wrap w-full gap-y-6",
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
                value: conf.read().colorspace,
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.colorspace = new_value;
                    });
                }
            }

            Dropdown {
                name: "Fallback Generator",
                class: "w-1/2 py-4 px-8",
                options: fallback_generator,
                value: conf.read().fallback_generator,
                onchange: move |new_value| {
                    conf.with_mut(|conf| {
                        conf.fallback_generator = new_value;
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
                class: "w-1/2 py-4 px-8",
                Button {
                    class: "rounded-md px-5 py-2 w-full text-sm font-semibold justify-center text-white shadow-sm !bg-indigo-600 hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer",
                    onclick: move |_| {
                        conf.set(WallustConfig::from_args_str(&wallpapers.read().source.wallust));
                    },
                    "Reset"
                }
            }

            div {
                class: "w-1/2 py-4 px-8",
                Button {
                    class: "rounded-md px-5 py-2 w-full text-sm font-semibold justify-center text-white shadow-sm !bg-indigo-600 hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 cursor-pointer",
                    onclick: move |_| {
                        conf.read().preview(&wallpapers.read().current.path());
                    },
                    "Preview"
                }
            }
        }
    }
}
