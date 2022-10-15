pub fn stafford_mix13(mut z: i64) -> i64 {
    z = (z ^ (z as u64 >> 30) as i64) * 0xBF58476D1CE4E5B9u64 as i64;
    z = (z ^ (z as u64 >> 27) as i64) * 0x94D049BB133111EBu64 as i64;

    z ^ (z as u64 >> 31) as i64
}

const PHI: i64 = 0x9E3779B97F4A7C15u64 as i64;

#[derive(Clone, Copy, Default)]
pub struct SodiumTextures {}

impl super::TextureProvider for SodiumTextures {
    fn random(&self, mut seed: i64) -> i32 {
        seed ^= ((seed as u64) >> 33) as i64;
        seed *= 0xff51afd7ed558ccdu64 as i64;
        seed ^= ((seed as u64) >> 33) as i64;
        seed *= 0xc4ceb9fe1a85ec53u64 as i64;
        seed ^= ((seed as u64) >> 33) as i64;

        seed += PHI;
        let rand1: i64 = stafford_mix13(seed);
        let rand2: i64 = stafford_mix13(seed + PHI);

        (rand1 + rand2) as i32
    }
}
