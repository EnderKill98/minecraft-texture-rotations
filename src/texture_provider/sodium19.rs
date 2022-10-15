#[derive(Clone, Copy, Default)]
pub struct Sodium19Textures {}

impl super::TextureProvider for Sodium19Textures {
    fn random(&self, seed: i64) -> i32 {
        let mut l: i64 = seed ^ 7640891576956012809i64;
        let mut m: i64 = l + -7046029254386353131i64;

        l = super::sodium::stafford_mix13(l); //lo
        m = super::sodium::stafford_mix13(m); //hi

        ((l + m).rotate_left(17) + l) as i32
    }
}
