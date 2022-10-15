mod placement;
mod rotation_info;
mod texture_finder;
mod texture_provider;

use crate::{
    rotation_info::RotationInfo,
    texture_provider::{Sodium19Textures, SodiumTextures, TextureProvider, VanillaTextures},
};
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser)]
struct Opts {
    /// Logging level (e.g. DEBUG, INFO, WARN or ERROR). You can also use the env RUST_LOG instead.
    #[clap(long, short)]
    level: Option<String>,

    /// Path to the toml config which specifies scanning parameters. See config.toml.sample for the format
    #[clap(long, short, default_value = "config.toml")]
    config: PathBuf,
    /*x: i32,
    y: i32,
    z: i32,
    modulo: i32,*/
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
    z_min: i32,
    z_max: i32,
    threads: i32,
    pin_threads_to_cores: bool,
    textures: String,
    formation: Vec<RotationInfo>,
}

fn main() {
    // Parse cli arguments
    let ref opts = Opts::parse();

    /*
    if true {
        #[rustfmt::skip]
        println!("Res Van: {}", VanillaTextures {}.get_texture(opts.x, opts.y, opts.z, opts.modulo));
        #[rustfmt::skip]
        println!("Res Sod: {}", SodiumTextures {}.get_texture(opts.x, opts.y, opts.z, opts.modulo));
        #[rustfmt::skip]
        println!("Res Sod19: {}", Sodium19Textures {}.get_texture(opts.x, opts.y, opts.z, opts.modulo));
        std::process::exit(0);
    }*/

    // Set logging level if RUST_LOG
    if let Some(level) = &opts.level {
        std::env::set_var("RUST_LOG", level);
    } else if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }

    // Initialize logger
    env_logger::builder().format_timestamp_millis().init();

    let config_path = &opts.config;
    if !config_path.exists() || config_path.is_dir() {
        log::error!(
            "Failed to load config ({config_path:?}). The file doesn't exist or is not a directory!"
        );
        std::process::exit(1);
    }

    let config_content = std::fs::read_to_string(config_path).expect("Reading toml config failed");
    let mut config: Config = toml::from_str(&config_content).expect("Parsing toml config failed");

    // Sanity checks
    assert!(config.x_min < config.x_max);
    assert!(config.y_min < config.y_max);
    assert!(config.z_min < config.z_max);

    // Fix rotation values
    for rotation_info in &mut config.formation {
        rotation_info.fix_rotation();
    }

    // Select texture provider
    match config.textures.as_str() {
        "Sodium" | "Sodium19" | "Vanilla" => {}
        _ => {
            log::error!(
                "Failed to select texture provider based on textures. Only \"Sodium\", \"Sodium19\" and \"Vanilla\" are supported."
            );
            std::process::exit(1);
        }
    };

    log::debug!("Config: {config:#?}");
    let placement = placement::Placement::new(&config.formation);

    let x_total: i32 = config.x_max - config.x_min;
    let per_x: i32 = x_total / config.threads;

    // Thread pinning
    let mut core_ids = if config.pin_threads_to_cores {
        Some(core_affinity::get_core_ids().unwrap())
    } else {
        None
    };

    // Warn if less cores available then threads specified
    if let Some(core_ids) = &core_ids {
        if core_ids.len() < config.threads as usize {
            log::warn!("You have specified more threads than available cores on the system. This is inefficient.");
        }
    }

    // Create threads
    let mut thread_handles = vec![];
    for (i, start) in (config.x_min..config.x_max)
        .step_by(per_x as usize + 1)
        .enumerate()
    {
        log::debug!("Worker {i} doing {start} to {}", start + per_x);
        let placement = placement.clone();
        let config = config.clone();

        let core_id = core_ids.as_mut().map(|ids| ids[i % ids.len()]);
        thread_handles.push(
            std::thread::Builder::new()
                .name(format!("Worker-{i:02}"))
                .spawn(move || {
                    if let Some(core_id) = core_id {
                        core_affinity::set_for_current(core_id);
                        log::debug!(
                            "Pinned thread {:?} to cpu {:?}",
                            std::thread::current().name().unwrap(),
                            core_id
                        );
                    }
                    match config.textures.as_str() {
                        "Sodium" => texture_finder::TextureFinder {
                            start_x: start,
                            end_x: start + per_x,
                            y_min: config.y_min,
                            y_max: config.y_max,
                            z_min: config.z_min,
                            z_max: config.z_max,
                            textures: SodiumTextures {},
                            placement,
                        }
                        .run(),
                        "Sodium19" => texture_finder::TextureFinder {
                            start_x: start,
                            end_x: start + per_x,
                            y_min: config.y_min,
                            y_max: config.y_max,
                            z_min: config.z_min,
                            z_max: config.z_max,
                            textures: Sodium19Textures {},
                            placement,
                        }
                        .run(),
                        "Vanilla" => texture_finder::TextureFinder {
                            start_x: start,
                            end_x: start + per_x,
                            y_min: config.y_min,
                            y_max: config.y_max,
                            z_min: config.z_min,
                            z_max: config.z_max,
                            textures: VanillaTextures {},
                            placement,
                        }
                        .run(),
                        _ => panic!("Unknown name!"),
                    };
                })
                .unwrap(),
        );
    }

    // Wait for all threads to finish
    thread_handles
        .into_iter()
        .for_each(|handle| handle.join().unwrap());
}
