#[allow(dead_code)]
#[path = "src/lib/cli.rs"]
mod cli;

use clap::CommandFactory;
use clap_mangen::Man;
use std::{fs, path::PathBuf};

fn generate_man_pages() -> Result<(), Box<dyn std::error::Error>> {
    let wallfacer_cmd = cli::WallfacerArgs::command();
    let man_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/man");
    fs::create_dir_all(&man_dir)?;

    // main wallfacer man page
    let mut buffer = Vec::default();
    Man::new(wallfacer_cmd.clone()).render(&mut buffer)?;
    fs::write(man_dir.join("wallfacer.1"), buffer)?;

    // subcommand man pages
    for subcmd in wallfacer_cmd.get_subcommands().filter(|c| !c.is_hide_set()) {
        let subcmd_name = format!("wallfacer-{}", subcmd.get_name());
        let subcmd = subcmd.clone().name(&subcmd_name);

        let mut buffer = Vec::default();

        Man::new(subcmd)
            .title(subcmd_name.to_uppercase())
            .render(&mut buffer)?;

        fs::write(man_dir.join(subcmd_name + ".1"), buffer)?;
    }

    Ok(())
}

fn main() {
    // override with the version passed in from nix
    // https://github.com/rust-lang/cargo/issues/6583#issuecomment-1259871885
    if let Ok(val) = std::env::var("NIX_RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={val}");
    }
    println!("cargo:rerun-if-env-changed=NIX_RELEASE_VERSION");

    if let Err(err) = generate_man_pages() {
        println!("cargo:warning=Error generating man pages: {err}");
    }
}
