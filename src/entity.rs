use crate::{
    geometry::{Cube, FineDirection, Quadrant, Sphere, Vec3, NB_QUADRANTS},
    matter_tree::CellPart,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityData {
    // TODO
    Creature,
    Voxels(Box<crate::voxel_grid::VoxelGridSpace>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    // This position is relative to the quadrant containing the center of the sphere
    // TODO Build the algorithm allowing comparing entities from different scales (iteratively
    // reconstructing the distance between the 2 entities origin quadrant gap, without overflowing
    // the temporary i64s
    pub bounding_sphere: Sphere,
    pub mass: i64,

    // TODO Keep bounding sphere and mass in sync with entity changes (mass changes & size changes
    // => Voxel tree growing / shrinking => changing sphere center & radius)
    pub entity: EntityData,
}

impl Entity {
    pub fn new(bounding_sphere: Sphere, entity: EntityData) -> Self {
        // TODO Get the entity mass
        Self {
            bounding_sphere,
            mass: 0,
            entity,
        }
    }
}

pub type EntityId = u64;

impl Entity {
    pub fn get_touched_external_cells(&self, area: &Cube) -> Vec<FineDirection> {
        let half_size = Vec3 {
            x: area.size / 2,
            y: area.size / 2,
            z: area.size / 2,
        };
        let area_center = area.origin.add(&half_size);
        let area_size = area.size;
        let relative_sphere_center = self.bounding_sphere.center.sub(&area_center);
        let radius = self.bounding_sphere.radius;
        // Early exit
        if relative_sphere_center.is_inside_centered_cube(area_size - radius) {
            return vec![];
        }

        // TODO
        vec![]
    }

    pub fn get_containing_cell_part(&self, area: &Cube) -> CellPart {
        let half_size = Vec3 {
            x: area.size / 2,
            y: area.size / 2,
            z: area.size / 2,
        };
        let area_center = area.origin.add(&half_size);
        let area_size = area.size;
        let relative_sphere = self.bounding_sphere.sub_to_center(&area_center);
        if !relative_sphere.center.is_inside_centered_cube(area_size) {
            return CellPart::CenterOutside;
        }
        if !relative_sphere
            .center
            .is_inside_centered_cube(area_size - relative_sphere.radius)
        {
            return CellPart::PartlyOutside;
        }

        for i in 0..NB_QUADRANTS {
            if relative_sphere.is_inside_quadrant(area_size, i) {
                return CellPart::Quadrant(num::FromPrimitive::from_usize(i).unwrap());
            }
        }
        CellPart::MultiQuadrant
    }

    pub fn get_collisioned_quadrants(&self, area: &Cube) -> Vec<u8> {
        let half_size = Vec3 {
            x: area.size / 2,
            y: area.size / 2,
            z: area.size / 2,
        };
        let area_center = area.origin.add(&half_size);
        let area_size = area.size;
        let relative_sphere_center = self.bounding_sphere.center.sub(&area_center);
        let radius = self.bounding_sphere.radius;
        let mut ret = vec![];
        for i in 0..NB_QUADRANTS {
            let shift = Vec3 {
                x: (i & (1 << 2)) as i64,
                y: (i & (1 << 2)) as i64,
                z: (i & (1 << 2)) as i64,
            }
            .mul_scalar(area_size)
            .sub(&half_size);
            let shifted_center = relative_sphere_center.sub(&shift);
            if shifted_center.is_inside_centered_cube(area_size / 2 + radius) {
                ret.push(i as u8);
            }
        }
        ret
    }

    pub fn switch_space_tree(&mut self, direction: Vec3, cell_size: i64) {
        self.bounding_sphere.center = self
            .bounding_sphere
            .center
            .sub(&direction.mul_scalar(cell_size));
    }

    pub fn check_collision(&self, other: &mut Self) -> bool {
        // TODO
        false
    }

    pub fn apply_collision(&mut self, other: &mut Self) {
        if !self.check_collision(other) {
            return;
        }

        // TODO
    }
}
