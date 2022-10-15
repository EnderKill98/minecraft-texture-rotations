use crate::rotation_info::RotationInfo;

// TODO: Remove need for clone
#[derive(Debug, Clone)]
pub struct Placement {
    pub tops_and_bottoms: Vec<RotationInfo>,
    pub sides: Vec<RotationInfo>,
}

impl Placement {
    pub fn new(formation: &[RotationInfo]) -> Self {
        let (mut tops_and_bottoms, mut sides) = (vec![], vec![]);
        for info in formation {
            if info.is_side {
                sides.push(*info);
            } else {
                tops_and_bottoms.push(*info);
            }
        }
        Self {
            tops_and_bottoms,
            sides,
        }
    }
}
