mod placement;
mod rotation_info;
mod texture_finder;
mod texture_provider;

use crate::rotation_info::RotationInfo;

// ------------------------------
// Config
use texture_provider::vanilla as selected_provider;

const RADIUS: i32 = 10000;
const X_MIN: i32 = -RADIUS;
const X_MAX: i32 = RADIUS;
const Z_MIN: i32 = -RADIUS;
const Z_MAX: i32 = RADIUS;
const Y_MIN: i32 = 60;
const Y_MAX: i32 = 140;
const THREADS: i32 = 16;
const PIN_THREADS_TO_CORES: bool = false;

const FORMATION: &[RotationInfo] = &[
    // Add formations here like this
    RotationInfo::new(-6, 1, 0, 3, false),
];
// ------------------------------

fn main() {
    const X_TOTAL: i32 = X_MAX - X_MIN;
    const PER_X: i32 = X_TOTAL / THREADS;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::builder().format_timestamp_millis().init();
    log::info!("X_MIN: {X_MIN}");
    log::info!("X_MAX: {X_MAX}");
    log::info!("Y_MIN: {Y_MIN}");
    log::info!("Y_MAX: {Y_MAX}");
    log::info!("Z_MIN: {Z_MIN}");
    log::info!("Z_MAX: {Z_MAX}");
    log::info!("FORMATION: {FORMATION:#?}");
    log::info!("THREADS: {THREADS}");

    let placement = placement::Placement::new(FORMATION);

    // Thread pinning
    let mut core_ids = if PIN_THREADS_TO_CORES {
        Some(core_affinity::get_core_ids().unwrap())
    } else {
        None
    };
    if let Some(core_ids) = &core_ids {
        if core_ids.len() < THREADS as usize {
            log::warn!("You specified more threads than cores! This is probably not efficient.");
        }
    }

    // Create threads
    let mut thread_handles = vec![];
    for (i, start) in (X_MIN..X_MAX).step_by(PER_X as usize + 1).enumerate() {
        let texture_finder = texture_finder::TextureFinder {
            placement: placement.clone(),
            start_x: start,
            end_x: start + PER_X,
        };
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
                    texture_finder.run()
                })
                .unwrap(),
        );
    }

    // Wait for all threads to finish
    thread_handles
        .into_iter()
        .for_each(|handle| handle.join().unwrap());
}
