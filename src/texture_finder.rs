use std::collections::HashSet;
use std::time::Instant;

use crate::cubiomes::{BiomeCache, BiomeID, CubiomesFinder};
use crate::{placement::Placement, texture_provider::TextureProvider};

pub struct TextureFinder<T> {
    pub start_x: i32,
    pub end_x: i32,
    pub y_min: i32,
    pub y_max: i32,
    pub z_min: i32,
    pub z_max: i32,
    pub textures: T,
    pub biome_filter: Option<(CubiomesFinder, HashSet<BiomeID>)>,
    pub biome_cache: Option<BiomeCache>,
    pub biome_cache_probe_count: u32,
    pub placement: Placement,
}

impl<T: TextureProvider> TextureFinder<T> {
    pub fn get_cached_biome_at(&mut self, x: i32, z: i32) -> BiomeID {
        let filter = self.biome_filter.as_ref().unwrap();
        if self.biome_cache.is_none() || !self.biome_cache.as_ref().unwrap().is_in_bounds(x, z) {
            // Update cache
            let wanted_elements = 4000000; // So we get a cache of roughly 16 MiB
            let sz = (self.z_max - self.z_min) + 1;
            let sx = (wanted_elements / sz)
                .max(1)
                .min((self.end_x - self.start_x) + 1);

            let thread_name = std::thread::current()
                .name()
                .unwrap_or("Unnamed Thread")
                .to_owned();
            /*log::debug!(
                "[{thread_name}] Generating biome cache for an {sx}x{sz} area ({} blocks total, last cache had {} probes)...",
                sx * sz, self.biome_cache_probe_count
            );*/
            log::debug!("[{thread_name}] Generating biome cache for an {sx}x{sz} area...",);
            let start = Instant::now();
            self.biome_cache = Some(BiomeCache::new(&filter.0, x, self.z_min, sx, sz));
            self.biome_cache_probe_count = 0;
            log::debug!(
                "[{thread_name}] Generated biome cache in {:?}",
                start.elapsed()
            );
        }

        self.biome_cache_probe_count += 1;
        self.biome_cache.as_ref().unwrap().get_biome_at(x, z)
    }

    pub fn run(&mut self) {
        let thread_name = std::thread::current()
            .name()
            .unwrap_or("Unnamed Thread")
            .to_owned();
        log::debug!(
            "[{}] Will scan from X {} to {} (inclusive)",
            thread_name,
            self.start_x,
            self.end_x,
        );

        let first = Instant::now();

        for x in self.start_x..=self.end_x {
            if x % 1000 == 0 {
                let max = self.end_x - self.start_x;
                let cur = x - self.start_x;
                log::debug!("[{}] Progress: {}%", thread_name, cur * 100 / max);
            }
            for z in self.z_min..=self.z_max {
                let biome_id = if self.biome_filter.is_some() {
                    let biome_id = self.get_cached_biome_at(x, z);
                    if !self.biome_filter.as_ref().unwrap().1.contains(&biome_id) {
                        continue;
                    }
                    Some(biome_id)
                } else {
                    None
                };

                'next_attempt: for y in self.y_min..=self.y_max {
                    for b in &self.placement.tops_and_bottoms {
                        if b.rotation != self.textures.get_texture(x + b.x, y + b.y, z + b.z, 4) {
                            continue 'next_attempt;
                        }
                    }
                    for b in &self.placement.sides {
                        if b.rotation != self.textures.get_texture(x + b.x, y + b.y, z + b.z, 2) {
                            continue 'next_attempt;
                        }
                    }

                    log::info!(
                        "[{thread_name}] Found at X: {x}, Y: {y}, Z: {z}{}",
                        if let Some(biome_id) = biome_id {
                            format!(" (biome {biome_id})")
                        } else {
                            String::new()
                        }
                    );
                }
            }
        }

        log::debug!("[{thread_name}] Finished after {:?}", first.elapsed());
    }

    pub fn run_with_tolerance(&mut self, max_failures: usize) {
        let thread_name = std::thread::current()
            .name()
            .unwrap_or("Unnamed Thread")
            .to_owned();
        log::debug!(
            "[{}] Will scan from X {} to {} (inclusive). Tolerating up to {} failures.",
            thread_name,
            self.start_x,
            self.end_x,
            max_failures,
        );

        let first = Instant::now();

        for x in self.start_x..=self.end_x {
            if x % 1000 == 0 {
                let max = self.end_x - self.start_x;
                let cur = x - self.start_x;
                log::debug!("[{}] Progress: {}%", thread_name, cur * 100 / max);
            }
            for z in self.z_min..=self.z_max {
                let biome_id = if self.biome_filter.is_some() {
                    let biome_id = self.get_cached_biome_at(x, z);
                    if !self.biome_filter.as_ref().unwrap().1.contains(&biome_id) {
                        continue;
                    }
                    Some(biome_id)
                } else {
                    None
                };
                'next_attempt: for y in self.y_min..=self.y_max {
                    let mut fails: usize = 0;
                    for b in &self.placement.tops_and_bottoms {
                        if b.rotation != self.textures.get_texture(x + b.x, y + b.y, z + b.z, 4) {
                            fails += 1;
                            if fails > max_failures {
                                continue 'next_attempt;
                            }
                        }
                    }
                    for b in &self.placement.sides {
                        if b.rotation != self.textures.get_texture(x + b.x, y + b.y, z + b.z, 2) {
                            fails += 1;
                            if fails > max_failures {
                                continue 'next_attempt;
                            }
                        }
                    }

                    log::info!(
                        "[{thread_name}] Found at X: {x}, Y: {y}, Z: {z} ({fails} fails{})",
                        if let Some(biome_id) = biome_id {
                            format!(", biome {biome_id}")
                        } else {
                            String::new()
                        }
                    );
                }
            }
        }

        log::debug!("[{thread_name}] Finished after {:?}", first.elapsed());
    }
}
