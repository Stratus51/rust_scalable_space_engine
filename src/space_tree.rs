use crate::entity::Entity;
use crate::geometry::{FineDirection, Quadrant, NB_QUADRANTS};
use crate::matter_tree::{CellPart, MatterTree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceTree {
    Parent(SpaceTreeParent),
    Matter(MatterTree),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceTreeParent {
    pub scale: u32,
    pub sub_trees: [Option<Box<SpaceTree>>; NB_QUADRANTS],
}

enum QuadrantMoveOperation {
    ToSubCell { quadrant: Quadrant },
    ToUpperCell,
}

struct EntityToDisplaceUp {
    path: Vec<Quadrant>,
    direction: FineDirection,
    entity: Box<Entity>,
}

struct EntityToDisplaceDown {
    path: Vec<Quadrant>,
    entity: Box<Entity>,
}

impl SpaceTree {
    const NONE_SPACE_CELL: Option<Box<Self>> = None;

    pub fn new_parent(child: Box<Self>, quadrant: Quadrant) -> Self {
        let scale = match child.as_ref() {
            Self::Parent(child) => child.scale + 1,
            Self::Matter(_) => 0,
        };
        let mut sub_trees = [Self::NONE_SPACE_CELL; NB_QUADRANTS];
        sub_trees[quadrant as usize] = Some(child);
        Self::Parent(SpaceTreeParent { scale, sub_trees })
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Parent(parent) => parent.sub_trees.iter().all(|cell| cell.is_none()),
            Self::Matter(matter) => matter.is_empty(),
        }
    }

    pub fn refresh(&mut self) -> Vec<EntityToDisplaceUp> {}
}
