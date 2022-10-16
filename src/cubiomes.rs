use crate::libcubiomes;
use std::ffi::{c_int, c_void};

pub use libcubiomes::BiomeID;

pub struct CubiomesFinder {
    generator: libcubiomes::Generator,
}

impl CubiomesFinder {
    pub fn new(seed: i64) -> Self {
        unsafe {
            let mut finder = CubiomesFinder {
                generator: libcubiomes::makeGenerator(),
            };
            libcubiomes::setupGenerator(
                &mut finder.generator,
                libcubiomes::MCVersion_MC_1_19 as c_int,
                0,
            );
            libcubiomes::applySeed(
                &mut finder.generator,
                libcubiomes::Dimension_DIM_OVERWORLD,
                seed as u64,
            );
            finder
        }
    }

    pub fn get_biome_at(&self, x: i32, y: i32, z: i32) -> BiomeID {
        unsafe {
            libcubiomes::getBiomeAt(&self.generator, 1 /* = Block coords */, x, y, z)
        }
    }
}

/// https://github.com/Cubitect/cubiomes/tree/master#biome-generation-in-a-range
pub struct BiomeCache {
    x: i32,
    y: i32,
    z: i32,
    sx: i32,
    sy: i32,
    sz: i32,
    biome_id_cache: *mut BiomeID,
}

impl BiomeCache {
    pub fn new(finder: &CubiomesFinder, x: i32, z: i32, sx: i32, sz: i32) -> Self {
        unsafe {
            let y = 63;
            let sy = 1;
            let r = libcubiomes::Range {
                scale: 16, // 1:16, a.k.a. horizontal chunk scaling
                // Define the position and size for a horizontal area:
                x,
                z,  // position (x,z)
                sx, // size (width,height)
                sz,
                // Set the vertical range as a plane near sea level at scale 1:4.
                y,
                sy,
            };
            let biome_id_cache: *mut BiomeID = libcubiomes::allocCache(&finder.generator, r);
            libcubiomes::genBiomes(&finder.generator, biome_id_cache, r);
            BiomeCache {
                biome_id_cache,
                x,
                y,
                z,
                sx,
                sy,
                sz,
            }
        }
    }
    pub fn is_in_bounds(&self, x: i32, z: i32) -> bool {
        //!(x >= self.x + self.sx || x < self.x || z >= self.z + self.sz || z < self.z)
        x >= self.x && x < self.x + self.sx && z >= self.z && z < self.z + self.sz
    }

    pub fn get_biome_at(&self, x: i32, z: i32) -> BiomeID {
        if !self.is_in_bounds(x, z) {
            panic!("Coordinate out of range for cache!");
        }
        let i_x = x - self.x;
        let i_y = 0;
        let i_z = z - self.z;
        let offset = i_y * self.sx * self.sz + i_z * self.sx + i_x;
        unsafe { *self.biome_id_cache.offset(offset as isize) }
    }
}

impl Drop for BiomeCache {
    fn drop(&mut self) {
        unsafe {
            libcubiomes::free(self.biome_id_cache as *mut c_void);
        }
    }
}
