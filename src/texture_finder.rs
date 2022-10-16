use crate::{placement::Placement, texture_provider::TextureProvider};

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
    pub fn run(&self) {
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

        let first = std::time::Instant::now();

        for x in self.start_x..=self.end_x {
            if x % 1000 == 0 {
                let max = self.end_x - self.start_x;
                let cur = x - self.start_x;
                log::debug!("[{}] Progress: {}%", thread_name, cur * 100 / max);
            }
            for z in self.z_min..=self.z_max {
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

                    log::info!("[{thread_name}] Found at X: {x}, Y: {y}, Z: {z}");
                }
            }
        }

        log::debug!("[{thread_name}] Finished after {:?}", first.elapsed());
    }

    pub fn run_with_tolerance(&self, max_failures: usize) {
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

        let first = std::time::Instant::now();

        for x in self.start_x..=self.end_x {
            if x % 1000 == 0 {
                let max = self.end_x - self.start_x;
                let cur = x - self.start_x;
                log::debug!("[{}] Progress: {}%", thread_name, cur * 100 / max);
            }
            for z in self.z_min..=self.z_max {
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

                    log::info!("[{thread_name}] Found at X: {x}, Y: {y}, Z: {z} ({fails} fails)");
                }
            }
        }

        log::debug!("[{thread_name}] Finished after {:?}", first.elapsed());
    }
}
