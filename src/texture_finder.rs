use crate::placement::Placement;
use crate::texture_provider::*;

pub struct TextureFinder {
    pub placement: Placement,
    pub start_x: i32,
    pub end_x: i32,
}

impl TextureFinder {
    pub fn run(&self) {
        let thread_name = std::thread::current()
            .name()
            .unwrap_or("Unnamed Thread")
            .to_owned();
        let first = std::time::Instant::now();

        for x in self.start_x..=self.end_x {
            if x % 1000 == 0 {
                let max = self.end_x - self.start_x;
                let cur = x - self.start_x;
                log::debug!("[{}] Progress: {}%", thread_name, cur * 100 / max);
            }
            for z in crate::Z_MIN..=crate::Z_MAX {
                'next_attempt: for y in crate::Y_MIN..=crate::Y_MAX {
                    for b in &self.placement.tops_and_bottoms {
                        if b.rotation != get_texture(x + b.x, y + b.y, z + b.z, 4) {
                            continue 'next_attempt;
                        }
                    }
                    for b in &self.placement.sides {
                        if b.rotation != get_texture(x + b.x, y + b.y, z + b.z, 2) {
                            continue 'next_attempt;
                        }
                    }

                    log::info!("[{thread_name}] Found at X: {x}, Y: {y}, Z: {z}");
                }
            }
        }

        log::debug!("[{thread_name}] Finished after {:?}", first.elapsed());
    }
}
