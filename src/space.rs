use crate::{
    geometry::Quadrant,
    matter_tree::{CellLocalisable, MatterTree},
    space_entity::SpaceEntity,
    space_tree::SpaceTree,
};

pub const SPACE_CELL_SIZE_POW: i64 = 20;
pub const SPACE_CELL_SIZE: i64 = 1 << SPACE_CELL_SIZE_POW; // ~ 1_000_000
pub const TICK_DIV: i64 = 1_000_000;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpaceConfiguration {
    pub tick_size: i64, // duration of a world tick in us

    // Optimization related
    pub gravity_threshold: u32, // cm/s^2
}

#[derive(Debug, PartialEq, Eq)]
pub struct Space {
    pub conf: SpaceConfiguration,

    // Data
    pub universe: Box<SpaceTree>,
}

impl Space {
    pub fn new(conf: SpaceConfiguration) -> Self {
        Self {
            conf,
            universe: Box::new(SpaceTree::new(0)),
        }
    }

    pub fn insert_entity(&mut self, mut entity: Box<SpaceEntity>) -> Vec<Quadrant> {
        let mut scale_up = vec![];
        let radius = entity.bounding_sphere.radius;
        // Expand universe until entity is in it
        while !entity
            .bounding_sphere
            .center
            .is_inside_centered_cube(SPACE_CELL_SIZE as i64 - radius)
        {
            // Enlarge universe
            let quadrant = entity.bounding_sphere.center.get_quadrant().invert();
            let new_universe = Box::new(SpaceTree::new(self.universe.scale + 1));
            let sub_cell = std::mem::replace(&mut self.universe, new_universe);
            self.universe.sub_trees[quadrant as usize] = Some(sub_cell);

            // Adjust entity
            entity.shrink(quadrant);

            // Save scale up
            scale_up.push(quadrant);
        }
        self.universe.add_entity(entity);
        scale_up
    }

    pub fn run(&mut self) {
        let outsiders = self.universe.refresh();
        let mut scale_up = vec![];
        for mut outsider in outsiders.into_iter() {
            for quadrant in scale_up.iter() {
                outsider.shrink(*quadrant);
            }
            scale_up.extend(self.insert_entity(outsider));
        }
    }
}
