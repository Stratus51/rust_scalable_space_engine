use crate::{
    geometry::{Quadrant, Sphere, Vec3, NB_QUADRANTS},
    matter_tree::{CellLocalisable, CellPart},
    space::SPACE_CELL_SIZE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceEntityData {
    // TODO
    Creature,
    Voxels(Box<crate::voxel_grid::VoxelGridSpace>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceEntity {
    // This position is relative to the quadrant containing the center of the sphere
    // TODO Build the algorithm allowing comparing entities from different scales (iteratively
    // reconstructing the distance between the 2 entities origin quadrant gap, without overflowing
    // the temporary i64s
    pub bounding_sphere: Sphere,
    pub mass: i64,

    // TODO Keep bounding sphere and mass in sync with entity changes (mass changes & size changes
    // => Voxel tree growing / shrinking => changing sphere center & radius)
    pub entity: SpaceEntityData,
}

impl SpaceEntity {
    pub fn new(bounding_sphere: Sphere, entity: SpaceEntityData) -> Self {
        // TODO Get the entity mass
        Self {
            bounding_sphere,
            mass: 0,
            entity,
        }
    }
}

impl CellLocalisable for SpaceEntity {
    fn get_containing_cell_part(&self, scale_factor: i64) -> CellPart {
        let bounding_sphere = self.bounding_sphere.div_scalar(scale_factor);
        if !bounding_sphere
            .center
            .is_inside_centered_cube(SPACE_CELL_SIZE - bounding_sphere.radius)
        {
            return CellPart::Outside;
        }

        for i in 0..NB_QUADRANTS {
            let shift = Vec3 {
                x: -SPACE_CELL_SIZE / 2 + ((i & (1 << 2)) as i64) * SPACE_CELL_SIZE,
                y: -SPACE_CELL_SIZE / 2 + ((i & (1 << 2)) as i64) * SPACE_CELL_SIZE,
                z: -SPACE_CELL_SIZE / 2 + ((i & (1 << 2)) as i64) * SPACE_CELL_SIZE,
            };
            let shifted_center = bounding_sphere.center.sub(&shift);
            if shifted_center.is_inside_centered_cube(SPACE_CELL_SIZE / 2 - bounding_sphere.radius)
            {
                return CellPart::Quadrant(num::FromPrimitive::from_usize(i).unwrap());
            }
        }
        CellPart::MultiQuadrant
    }

    fn shrink(&mut self, quadrant: Quadrant) {
        let shift = Vec3 {
            x: -SPACE_CELL_SIZE / 2 + (quadrant as i64 & (1 << 2)) * SPACE_CELL_SIZE,
            y: -SPACE_CELL_SIZE / 2 + (quadrant as i64 & (1 << 2)) * SPACE_CELL_SIZE,
            z: -SPACE_CELL_SIZE / 2 + (quadrant as i64 & (1 << 2)) * SPACE_CELL_SIZE,
        };
        self.bounding_sphere.center = self.bounding_sphere.center.div_scalar(2).add(&shift);
    }

    fn expand(&mut self, quadrant: Quadrant) {
        let shift = Vec3 {
            x: -SPACE_CELL_SIZE / 2 + (quadrant as i64 & (1 << 2)) * SPACE_CELL_SIZE,
            y: -SPACE_CELL_SIZE / 2 + (quadrant as i64 & (1 << 2)) * SPACE_CELL_SIZE,
            z: -SPACE_CELL_SIZE / 2 + (quadrant as i64 & (1 << 2)) * SPACE_CELL_SIZE,
        };
        self.bounding_sphere.center = self.bounding_sphere.center.sub(&shift).mul_scalar(2);
    }
}
