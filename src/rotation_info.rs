use serde::Deserialize;

/// Important: Call fix_rotation if not using new()!
/// Otherwise the rotation value will not be adjusted and wrong!
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Deserialize)]
pub struct RotationInfo {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rotation: i32,
    pub is_side: bool,
}

impl RotationInfo {
    #[allow(dead_code)]
    pub const fn new(x: i32, y: i32, z: i32, rotation: i32, is_side: bool) -> Self {
        Self {
            x,
            y,
            z,
            rotation: if is_side { rotation % 2 } else { rotation },
            is_side,
        }
    }

    pub fn fix_rotation(&mut self) {
        if self.is_side {
            self.rotation %= 2;
        }
    }
}
