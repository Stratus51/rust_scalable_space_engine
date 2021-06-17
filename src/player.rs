use crate::geometry::Vec3;

pub const MASS: f64 = 100.0;
pub const RADIUS: i64 = 200;
pub const CONTROL_FORCE: i64 = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub control_forces: Vec3,
    pub drop_block: bool,
    pub drop_block_fixed: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            control_forces: Vec3::ZERO,
            drop_block: false,
            drop_block_fixed: false,
        }
    }

    pub fn control(&mut self, dir: &Vec3) {
        let div = dir.length_f64();
        self.control_forces = dir.mul_scalar(CONTROL_FORCE).div_float(div);
    }
}
