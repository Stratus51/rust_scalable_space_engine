#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Vec3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Vec3 {
    pub fn zero() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    pub fn add(&self, other: &Vec3) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn mul_scalar(&self, v: i64) -> Self {
        Self {
            x: self.x * v,
            y: self.y * v,
            z: self.z * v,
        }
    }

    pub fn div_scalar(&self, v: i64) -> Self {
        Self {
            x: self.x / v,
            y: self.y / v,
            z: self.z / v,
        }
    }

    pub fn is_inside_centered_cube(&self, side_length: i64) -> bool {
        self.x.abs() < side_length && self.y.abs() < side_length && self.z.abs() < side_length
    }

    pub fn get_quadrant(&self) -> Quadrant {
        Quadrant::from_pos(self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Mat3 {
    pub divider: i64,
    pub values: [i64; 9],
}

impl Mat3 {
    pub fn identity() -> Self {
        Self {
            divider: 1,
            values: [1, 0, 0, 0, 1, 0, 0, 0, 1],
        }
    }

    pub fn mul_vec(&self, vec: &Vec3) -> Vec3 {
        Vec3 {
            x: (vec.x * self.values[0] + vec.y * self.values[1] + vec.z * self.values[2])
                / self.divider,
            y: (vec.x * self.values[3] + vec.y * self.values[4] + vec.z * self.values[5])
                / self.divider,
            z: (vec.x * self.values[6] + vec.y * self.values[7] + vec.z * self.values[8])
                / self.divider,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Xp = 0,
    Yp = 1,
    Zp = 2,
    Xn = 3,
    Yn = 4,
    Zn = 5,
}

#[repr(usize)]
#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Quadrant {
    XnYnZn = 0,
    XnYnZp = 1,
    XnYpZn = 2,
    XnYpZp = 3,
    XpYnZn = 4,
    XpYnZp = 5,
    XpYpZn = 6,
    XpYpZp = 7,
}

impl Quadrant {
    pub fn from_pos(pos: &Vec3) -> Self {
        let val = (pos.x >= 0) as usize * (1 << 2)
            + (pos.y >= 0) as usize * (1 << 1)
            + (pos.z >= 0) as usize;
        num::FromPrimitive::from_usize(val).unwrap()
    }

    pub fn invert(&self) -> Self {
        match self {
            Self::XnYnZn => Self::XpYpZp,
            Self::XnYnZp => Self::XpYpZn,
            Self::XnYpZn => Self::XpYnZp,
            Self::XnYpZp => Self::XpYnZn,
            Self::XpYnZn => Self::XnYpZp,
            Self::XpYnZp => Self::XnYpZn,
            Self::XpYpZn => Self::XnYnZp,
            Self::XpYpZp => Self::XnYnZn,
        }
    }
}

pub const NB_QUADRANTS: usize = 8;
