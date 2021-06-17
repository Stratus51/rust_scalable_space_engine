#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Vec3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

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

    pub fn div_float(&self, v: f64) -> Self {
        Self {
            x: (self.x as f64 / v) as i64,
            y: (self.y as f64 / v) as i64,
            z: (self.z as f64 / v) as i64,
        }
    }

    pub fn dot_f64(&self, other: &Self) -> f64 {
        self.x as f64 * other.x as f64
            + self.y as f64 * other.y as f64
            + self.z as f64 * other.z as f64
    }

    pub fn is_inside_centered_cube(&self, side_length: i64) -> bool {
        let min = -side_length / 2;
        let max = side_length / 2 - 1;
        (self.x >= min && self.x <= max)
            && (self.y >= min && self.y <= max)
            && (self.z >= min && self.z <= max)
    }

    pub fn get_quadrant(&self) -> Quadrant {
        Quadrant::from_pos(self)
    }

    pub fn direction_components(&self) -> Vec<Direction> {
        let mut ret = vec![];
        if self.x > 0 {
            ret.push(Direction::Xp);
        } else if self.x < 0 {
            ret.push(Direction::Xn);
        }
        if self.y > 0 {
            ret.push(Direction::Yp);
        } else if self.y < 0 {
            ret.push(Direction::Yn);
        }
        if self.z > 0 {
            ret.push(Direction::Zp);
        } else if self.z < 0 {
            ret.push(Direction::Zn);
        }
        ret
    }

    pub fn length_f64(&self) -> f64 {
        let x = self.x as f64;
        let y = self.y as f64;
        let z = self.z as f64;
        f64::sqrt(x * x + y * y + z * z)
    }

    pub fn remove_matching_quadrant_component(&self, quadrant: Quadrant) -> Self {
        let mut ret = *self;
        let quad_x_pos = quadrant.x_p();
        if (quad_x_pos && self.x > 0) || (!quad_x_pos && self.x < 0) {
            ret.x = 0;
        }
        let quad_y_pos = quadrant.y_p();
        if (quad_y_pos && self.y > 0) || !quad_y_pos && self.y < 0 {
            ret.y = 0;
        }
        let quad_z_pos = quadrant.z_p();
        if (quad_z_pos && self.z > 0) || (!quad_z_pos && self.z < 0) {
            ret.z = 0;
        }
        ret
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

pub const NB_DIRECTIONS: u8 = 6;

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

    pub fn outsider_direction_vec(pos: &Vec3, size: i64) -> Vec3 {
        let x = Self::component(pos.x, size / 2) as i64 - 1;
        let y = Self::component(pos.y, size / 2) as i64 - 1;
        let z = Self::component(pos.z, size / 2) as i64 - 1;
        Vec3 { x, y, z }
    }

    pub fn equivalent_vec(&self) -> Vec3 {
        let mut val = *self as i64;
        let x = val / 3 - 1;
        val %= 3;
        let y = val / 3 - 1;
        val %= 3;
        let z = val - 1;
        Vec3 { x, y, z }
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

    pub fn move_to(&self, direction: Vec3) -> Option<Self> {
        let x = self.x_p() as i64 + direction.x;
        let y = self.y_p() as i64 + direction.y;
        let z = self.z_p() as i64 + direction.z;
        if x > 1 || y > 1 || z > 1 || x < 0 || y < 0 || z < 0 {
            None
        } else {
            let val = (x << 2) + (y << 1) + z;
            Some(num::FromPrimitive::from_i64(val).unwrap())
        }
    }

    pub fn mirror(&self, direction: Vec3) -> Self {
        let x = self.x_p() ^ (direction.x != 0);
        let y = self.y_p() ^ (direction.y != 0);
        let z = self.z_p() ^ (direction.z != 0);
        let val = ((x as u8) << 2) + ((y as u8) << 1) + z as u8;
        num::FromPrimitive::from_u8(val).unwrap()
    }

    pub fn match_direction(&self, direction: Vec3) -> bool {
        let x = (self.x_p() && (direction.x >= 0)) || (!self.x_p() && (direction.x <= 0));
        let y = (self.y_p() && (direction.y >= 0)) || (!self.y_p() && (direction.y <= 0));
        let z = (self.z_p() && (direction.z >= 0)) || (!self.z_p() && (direction.z <= 0));
        x && y && z
    }
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

    pub fn add_to_center(&self, v: &Vec3) -> Self {
        Self {
            center: self.center.add(&v),
            radius: self.radius,
        }
    }

    pub fn sub_to_center(&self, v: &Vec3) -> Self {
        Self {
            center: self.center.sub(&v),
            radius: self.radius,
        }
    }

    pub fn move_by(&mut self, shift: &Vec3) {
        self.center = self.center.add(shift);
    }

    pub fn is_inside_quadrant(&self, cell_area: &Cube, quadrant: usize) -> bool {
        let half_size = cell_area.size / 2;
        let quarter_size = half_size / 2;
        let quarter_vec = Vec3 {
            x: quarter_size,
            y: quarter_size,
            z: quarter_size,
        };
        let shift = Vec3 {
            x: (quadrant as i64 & (1 << 2) != 0) as i64,
            y: (quadrant as i64 & (1 << 1) != 0) as i64,
            z: (quadrant as i64 & (1 << 0) != 0) as i64,
        }
        .mul_scalar(half_size)
        .sub(&quarter_vec);
        let shifted_center = self.center.sub(&shift);
        shifted_center.is_inside_centered_cube(half_size - self.radius)
    }

    pub fn intersects(&self, other: &Sphere) -> bool {
        let dist = self.center.sub(&other.center).length_f64();
        let limit_dist = self.radius + other.radius;
        dist < limit_dist as f64
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cube {
    pub origin: Vec3,
    pub size: i64,
}
