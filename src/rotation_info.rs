#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RotationInfo {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rotation: i32,
    pub is_side: bool,
}

impl RotationInfo {
    pub const fn new(x: i32, y: i32, z: i32, rotation: i32, is_side: bool) -> Self {
        Self {
            x,
            y,
            z,
            rotation: if is_side { rotation % 2 } else { rotation },
            is_side,
        }
    }
}
