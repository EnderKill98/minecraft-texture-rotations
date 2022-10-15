pub fn stafford_mix13(mut z: i64) -> i64 {
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9u64 as i64;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBu64 as i64;

    z ^ (z >> 31)
}

const PHI: i64 = 0x9E3779B97F4A7C15u64 as i64;

#[allow(dead_code)]
pub fn random(mut seed: i64) -> i32 {
    seed ^= seed >> 33;
    seed *= 0xff51afd7ed558ccdu64 as i64;
    seed ^= seed >> 33;
    seed *= 0xc4ceb9fe1a85ec53u64 as i64;
    seed ^= seed >> 33;

    seed += PHI;
    let rand1: i64 = stafford_mix13(seed);
    let rand2: i64 = stafford_mix13(seed + PHI);

    (rand1 + rand2) as i32
}
