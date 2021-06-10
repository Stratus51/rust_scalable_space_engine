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

    pub fn get_displaced_outsider(entity: Box<Entity>) -> EntityToDisplaceUp {
        EntityToDisplaceUp {
            path: vec![],
            direction: FineDirection::from_outsider_pos(
                &entity.bounding_sphere.center,
                MatterTree::MAX_SIZE,
            ),
            entity,
        }
    }

    pub fn add_entities(&mut self, entities: Vec<EntityToDisplaceDown>) {}

    pub fn refresh(&mut self) -> Vec<EntityToDisplaceUp> {
        match self {
            Self::Matter(cell) => {
                let outsiders = cell.refresh();
                outsiders
                    .into_iter()
                    .map(Self::get_displaced_outsider)
                    .collect()
            }
            Self::Parent(parent) => {
                let mut outsiders = vec![];
                let mut relocate = [vec![]; NB_QUADRANTS];
                for (i, child) in parent.sub_trees.iter_mut().enumerate() {
                    if let Some(child) = child {
                        let quadrant: Quadrant = num::FromPrimitive::from_usize(i).unwrap();
                        let sub_outsiders = child.refresh();
                        for displaced_outsider in sub_outsiders.into_iter() {
                            if let Some(relocation) = quadrant.move_to(displaced_outsider.direction)
                            {
                                relocate[relocation as usize].push(EntityToDisplaceDown {
                                    path: displaced_outsider.path,
                                    entity: displaced_outsider.entity,
                                });
                            } else {
                                displaced_outsider
                                    .path
                                    .push(quadrant.mirror(displaced_outsider.direction));
                                outsiders.push(displaced_outsider);
                            }
                        }
                    }
                }
                for (i, entities) in relocate.into_iter().enumerate() {
                    if parent.sub_trees[i].is_none() {
                        if parent.scale == 0 {
                            parent.sub_trees[i] = Some(Box::new(Self::Matter(MatterTree::new())));
                        } else {
                            parent.sub_trees[i] = Some(Box::new(Self::Parent(SpaceTreeParent {
                                scale: parent.scale - 1,
                                sub_trees: [Self::NONE_SPACE_CELL; NB_QUADRANTS],
                            })));
                        }
                    }
                    // TODO Is there a cleaner Rust way to write this?
                    let sub_tree = parent.sub_trees[i].unwrap();
                    sub_tree.add_entities(entities);
                }
                outsiders
            }
        }
    }
}
