mod sodium;
mod sodium19;
mod vanilla;

pub use sodium::SodiumTextures;
pub use sodium19::Sodium19Textures;
pub use vanilla::VanillaTextures;

pub trait TextureProvider: Copy + Default {
    fn get_coordinate_random(&self, x: i32, y: i32, z: i32) -> i64 {
        let mut l: i64 = (x as i64 * 3129871) ^ z as i64 * 116129781i64 ^ y as i64;
        l = l * l * 42317861i64 + l * 11i64;
        l >> 16
    }

    fn get_texture(&self, x: i32, y: i32, z: i32, modulo: i32) -> i32 {
        let rand: i32 = self.random(self.get_coordinate_random(x, y, z));
        rand.abs() % modulo
    }

    fn random(&self, seed: i64) -> i32;
}
