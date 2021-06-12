use crate::geometry::Vec3;

pub const MASS: f64 = 100.0;
pub const RADIUS: i64 = 200;
pub const CONTROL_FORCE: i64 = 10000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub control_forces: Vec3,
}

impl Player {
    pub fn new() -> Self {
        Self {
            control_forces: Vec3::ZERO,
        }
    }

    pub fn control(&mut self, dir: &Vec3) {
        let div = dir.distance();
        self.control_forces = dir.mul_scalar(CONTROL_FORCE).div_float(div);
        println!(
            "control: dir = {:?} | div = {} | force_val = {} | resulting_force = {:?}",
            dir, div, CONTROL_FORCE, self.control_forces
        );
    }
}
