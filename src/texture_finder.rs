use std::time::Instant;

use crate::{placement::Placement, texture_provider::TextureProvider};
//use cubiomes::finders::{BiomeCache, BiomeID, CoordScaling, CubiomesFinder};

pub struct TextureFinder<T> {
    pub start_x: i32,
    pub end_x: i32,
    pub y_min: i32,
    pub y_max: i32,
    pub z_min: i32,
    pub z_max: i32,
    pub textures: T,
    pub placement: Placement,
}

impl<T: TextureProvider> TextureFinder<T> {
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
                for mirror_xz in [false, true] {
                    'next_attempt: for y in self.y_min..=self.y_max {
                        for b in &self.placement.tops_and_bottoms {
                            let rel = (
                                if mirror_xz { -b.x } else { b.x },
                                if mirror_xz { -b.z } else { b.z },
                            );
                            if b.rotation
                                != (self.textures.get_texture(x + rel.0, y + b.y, z + rel.1, 4)
                                    + if mirror_xz { 2 } else { 0 })
                                    % 4
                            {
                                continue 'next_attempt;
                            }
                        }
                        for b in &self.placement.sides {
                            let rel = (
                                if mirror_xz { -b.x } else { b.x },
                                if mirror_xz { -b.z } else { b.z },
                            );
                            if b.rotation
                                != self.textures.get_texture(x + rel.0, y + b.y, z + rel.1, 2) % 4
                            {
                                continue 'next_attempt;
                            }
                        }

                        log::info!(
                            "[{thread_name}] Found at X: {x}, Y: {y}, Z: {z} (mirror_xz {mirror_xz})",
                        );
                    }
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
                for mirror_xz in [false, true] {
                    'next_attempt: for y in self.y_min..=self.y_max {
                        let mut fails: usize = 0;
                        for b in &self.placement.tops_and_bottoms {
                            let rel = (
                                if mirror_xz { -b.x } else { b.x },
                                if mirror_xz { -b.z } else { b.z },
                            );
                            if b.rotation
                                != (self.textures.get_texture(x + rel.0, y + b.y, z + rel.1, 4)
                                    + if mirror_xz { 2 } else { 0 })
                                    % 4
                            {
                                fails += 1;
                                if fails > max_failures {
                                    continue 'next_attempt;
                                }
                            }
                        }
                        for b in &self.placement.sides {
                            let rel = (
                                if mirror_xz { -b.x } else { b.x },
                                if mirror_xz { -b.z } else { b.z },
                            );
                            if b.rotation
                                != self.textures.get_texture(x + rel.0, y + b.y, z + rel.1, 2) % 4
                            {
                                fails += 1;
                                if fails > max_failures {
                                    continue 'next_attempt;
                                }
                            }
                        }

                        log::info!(
                            "[{thread_name}] Found at X: {x}, Y: {y}, Z: {z} ({fails} fails, mirror_xz {mirror_xz})",
                        );
                    }
                }
            }
        }

        log::debug!("[{thread_name}] Finished after {:?}", first.elapsed());
    }
    /*
    fn get_rotations_for_range(
        x_min: i32,
        x_max: i32,
        y_min: i32,
        y_max: i32,
        z_min: i32,
        z_max: i32,
    ) -> RotationCache {
    }*/
}
/*
struct RotationCache {
    cache: Vec<u8>,
    x: i32,
    x_size: usize,
    y: i32,
    y_size: usize,
    z: i32,
    z_size: usize,
}

impl RotationCache {
    fn index_of(&self, x: i32, y: i32, z: i32) -> usize {
        let i_x = (x - self.x) as usize;
        let i_y = (y - self.y) as usize;
        let i_z = (z - self.z) as usize;
        self.x_size * self.y_size * self.z_size * i_x + self.y_size + self.z_size * i_z + i_y
    }
    fn get_rotation_at(&self, x: i32, y: i32, z: i32) -> u8 {
        self.cache[self.index_of(x, y, z)]
    }
}*/
