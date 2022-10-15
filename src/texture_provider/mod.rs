pub mod sodium;
pub mod sodium19;
pub mod vanilla;

pub fn get_coordinate_random(x: i32, y: i32, z: i32) -> i64 {
    let mut l: i64 = (x as i64 * 3129871) ^ z as i64 * 116129781i64 ^ y as i64;
    l = l * l * 42317861i64 + l * 11i64;
    l >> 16
}

pub fn get_texture(x: i32, y: i32, z: i32, modulo: i32) -> i32 {
    let rand: i32 = crate::selected_provider::random(get_coordinate_random(x, y, z));
    rand.abs() % modulo
}
