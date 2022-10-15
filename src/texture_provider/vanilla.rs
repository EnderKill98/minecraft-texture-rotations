const MULTIPLIER: i64 = 0x5DEECE66Di64;
const MASK: i64 = (1i64 << 48) - 1;

#[allow(dead_code)]
pub fn random(seed: i64) -> i32 {
    let seed = (seed ^ MULTIPLIER) & MASK;
    ((seed * 0xBB20B4600A69i64 + 0x40942DE6BAi64) >> 16) as i32
}
