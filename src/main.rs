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
use std::{collections::HashSet, path::PathBuf};

pub const LO_SEED: i64 = 64149200;

#[derive(Parser)]
enum Command {
    Scan(ScanOpts),
    Verify(VerifyOpts),
}

#[derive(Parser)]
struct ScanOpts {
    /// Logging level (e.g. DEBUG, INFO, WARN or ERROR). You can also use the env RUST_LOG instead.
    #[clap(long, short)]
    log_level: Option<String>,

    ///// Optional path to a new file to write results additionally into
    //#[clap(long, short)]
    //output: Option<PathBuf>,
    //
    /// Allow up to the given amount of failures (scan will take longer!)
    #[clap(long, short = 'f')]
    max_failures: Option<usize>,

    ///// Optional path to a new file to write results additionally into
    //#[clap(long, short)]
    //output: Option<PathBuf>,
    /// Path to the toml config which specifies scanning parameters. See config.toml.sample for the format
    config: PathBuf,
}

/// Get the rotation value of a texture at a given coordinate for all the texture variants.
#[derive(Parser)]
struct VerifyOpts {
    // X block coordinate
    x: i32,
    // Y block coordinate
    y: i32,
    // Z block coordinate
    z: i32,
    // Used for the side values of certain textures
    #[clap(long, short = 's')]
    is_side: bool,
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
    filter_for_biome_ids: HashSet<cubiomes::finders::BiomeID>,
    formation: Vec<RotationInfo>,
}

fn main() {
    // Parse cli arguments
    match Command::parse() {
        Command::Scan(opts) => scan(opts),
        Command::Verify(opts) => verify(opts),
    }
}

fn verify(opts: VerifyOpts) {
    let modulo = if opts.is_side { 2 } else { 4 };
    let (x, y, z) = (opts.x, opts.y, opts.z);

    let sod = SodiumTextures {}.get_texture(x, y, z, modulo);
    let sod19 = Sodium19Textures {}.get_texture(x, y, z, modulo);
    let van = VanillaTextures {}.get_texture(x, y, z, modulo);

    println!("Rotation values at {x}, {y}, {z}{} are {sod} (Sodium), {sod19} (Sodium19) and {van} (Vanilla)", if opts.is_side { " (is side)" } else { "" });
}

fn scan(opts: ScanOpts) {
    // Set logging level if RUST_LOG
    if let Some(level) = &opts.log_level {
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

    //log::debug!("Config: {config:#?}");
    log::debug!("Using config {:?}:", config_path);
    log::debug!("  X: {} (min) to {} (max)", config.x_min, config.x_max);
    log::debug!("  Y: {} (min) to {} (max)", config.y_min, config.y_max);
    log::debug!("  Z: {} (min) to {} (max)", config.z_min, config.z_max);
    log::debug!("  {} threads", config.threads);
    if config.filter_for_biome_ids.len() > 0 {
        log::debug!("  Filtering for biomes: {:?}", config.filter_for_biome_ids);
    }
    log::debug!("  The formation has {} rotations", config.formation.len());
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

    // Check max_failures value
    if let Some(max_failures) = opts.max_failures {
        if max_failures == 0 {
            log::warn!(
                "Just remove this argument. You'll otherwise just waste resources for no gain. ;)"
            );
        }
        if max_failures >= config.formation.len() {
            log::error!("You shouldn't allow more failures then actual blocks in your formation!");
            std::process::exit(1);
        }
    }

    let max_failures = opts.max_failures;
    // Create threads
    let mut thread_handles = vec![];
    for (i, start) in (config.x_min..config.x_max)
        .step_by(per_x as usize + 1)
        .enumerate()
    {
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

                    let biome_filter = if config.filter_for_biome_ids.len() > 0 {
                        Some((
                            cubiomes::finders::CubiomesFinder::new(LO_SEED),
                            config.filter_for_biome_ids.clone(),
                        ))
                    } else {
                        None
                    };

                    match config.textures.as_str() {
                        "Sodium" => {
                            if let Some(max_failures) = max_failures {
                                texture_finder::TextureFinder {
                                    start_x: start,
                                    end_x: start + per_x,
                                    y_min: config.y_min,
                                    y_max: config.y_max,
                                    z_min: config.z_min,
                                    z_max: config.z_max,
                                    textures: SodiumTextures {},
                                    biome_filter,
                                    biome_cache: None,
                                    biome_cache_probe_count: 0,
                                    placement,
                                }
                                .run_with_tolerance(max_failures)
                            } else {
                                texture_finder::TextureFinder {
                                    start_x: start,
                                    end_x: start + per_x,
                                    y_min: config.y_min,
                                    y_max: config.y_max,
                                    z_min: config.z_min,
                                    z_max: config.z_max,
                                    textures: SodiumTextures {},
                                    biome_filter,
                                    biome_cache: None,
                                    biome_cache_probe_count: 0,
                                    placement,
                                }
                                .run()
                            }
                        }
                        "Sodium19" => {
                            if let Some(max_failures) = max_failures {
                                texture_finder::TextureFinder {
                                    start_x: start,
                                    end_x: start + per_x,
                                    y_min: config.y_min,
                                    y_max: config.y_max,
                                    z_min: config.z_min,
                                    z_max: config.z_max,
                                    textures: Sodium19Textures {},
                                    biome_filter,
                                    biome_cache: None,
                                    biome_cache_probe_count: 0,
                                    placement,
                                }
                                .run_with_tolerance(max_failures)
                            } else {
                                texture_finder::TextureFinder {
                                    start_x: start,
                                    end_x: start + per_x,
                                    y_min: config.y_min,
                                    y_max: config.y_max,
                                    z_min: config.z_min,
                                    z_max: config.z_max,
                                    textures: Sodium19Textures {},
                                    biome_filter,
                                    biome_cache: None,
                                    biome_cache_probe_count: 0,
                                    placement,
                                }
                                .run()
                            }
                        }
                        "Vanilla" => {
                            if let Some(max_failures) = max_failures {
                                texture_finder::TextureFinder {
                                    start_x: start,
                                    end_x: start + per_x,
                                    y_min: config.y_min,
                                    y_max: config.y_max,
                                    z_min: config.z_min,
                                    z_max: config.z_max,
                                    textures: VanillaTextures {},
                                    biome_filter,
                                    biome_cache: None,
                                    biome_cache_probe_count: 0,
                                    placement,
                                }
                                .run_with_tolerance(max_failures)
                            } else {
                                texture_finder::TextureFinder {
                                    start_x: start,
                                    end_x: start + per_x,
                                    y_min: config.y_min,
                                    y_max: config.y_max,
                                    z_min: config.z_min,
                                    z_max: config.z_max,
                                    textures: VanillaTextures {},
                                    biome_filter,
                                    biome_cache: None,
                                    biome_cache_probe_count: 0,
                                    placement,
                                }
                                .run()
                            }
                        }
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
