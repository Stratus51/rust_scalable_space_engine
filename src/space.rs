use crate::{
    entity::{Entity, EntityId},
    geometry::Quadrant,
    space_tree::SpaceTree,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Space {
    pub universe: Box<SpaceTree>,
}

impl Space {
    pub fn new() -> Self {
        Self {
            universe: Box::new(SpaceTree::new()),
        }
    }

    pub fn refresh(&mut self) {
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
