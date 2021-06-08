use crate::{
    entity::{Entity, EntityId},
    geometry::Quadrant,
    space_tree::SpaceTree,
};

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

    pub fn run(&mut self) {
        // Move objects
        // TODO LATER This should create a 4D volume for collision calculus instead of moving the
        // objects straight away. This might be too calculus heavy, so instead it could generate a
        // cylinder representing the sphere movement for collision calculus.

        // TODO Apply distant forces

        // TODO Apply link forces

        // TODO Fetch collisions

        // TODO Apply collisions
        // - destruction
        // - forces applied
        // - move to solve incoherency if necessary

        // Refresh Space Tree
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
