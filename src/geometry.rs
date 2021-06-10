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

    pub fn sub(&self, other: &Vec3) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
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
        let min = -side_length;
        let max = side_length - 1;
        (self.x > min || self.x < max)
            && (self.y > min || self.y < max)
            && (self.z > min || self.z < max)
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
    pub const IDENTITY: Self = Self {
        divider: 1,
        values: [1, 0, 0, 0, 1, 0, 0, 0, 1],
    };

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

#[repr(u8)]
#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FineDirection {
    XnYnZn = 0,
    XnYnZz = 1,
    XnYnZp = 2,
    XnYzZn = 3,
    XnYzZz = 4,
    XnYzZp = 5,
    XnYpZn = 6,
    XnYpZz = 7,
    XnYpZp = 8,
    XzYnZn = 9,
    XzYnZz = 10,
    XzYnZp = 11,
    XzYzZn = 12,
    XzYzZz = 13,
    XzYzZp = 14,
    XzYpZn = 15,
    XzYpZz = 16,
    XzYpZp = 17,
    XpYnZn = 18,
    XpYnZz = 19,
    XpYnZp = 20,
    XpYzZn = 21,
    XpYzZz = 22,
    XpYzZp = 23,
    XpYpZn = 24,
    XpYpZz = 25,
    XpYpZp = 26,
}

impl FineDirection {
    fn component(pos: i64, size: i64) -> u8 {
        if pos < -size {
            0
        } else if pos < size {
            1
        } else {
            2
        }
    }

    pub fn from_outsider_pos(pos: &Vec3, size: i64) -> Self {
        let x = Self::component(pos.x, size);
        let y = Self::component(pos.y, size);
        let z = Self::component(pos.z, size);
        let val = x * 3 * 3 + y * 3 + z;
        num::FromPrimitive::from_u8(val).unwrap()
    }
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
    pub fn x_p(&self) -> bool {
        *self as usize & (1 << 2) != 0
    }

    pub fn y_p(&self) -> bool {
        *self as usize & (1 << 1) != 0
    }

    pub fn z_p(&self) -> bool {
        *self as usize & (1 << 0) != 0
    }

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

    pub fn move_to(&self, direction: FineDirection) -> Option<Self> {}

    pub fn mirror(&self, direction: FineDirection) -> Self {}
}

pub const NB_QUADRANTS: usize = 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: i64,
}

impl Sphere {
    pub fn div_scalar(&self, value: i64) -> Self {
        Self {
            center: self.center,
            radius: self.radius / value,
        }
    }

    pub fn add_vector(&self, v: Vec3) -> Self {
        Self {
            center: self.center.add(&v),
            radius: self.radius,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cube {
    pub origin: Vec3,
    pub size: i64,
}
