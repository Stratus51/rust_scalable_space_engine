use crate::geometry::{Quadrant, NB_QUADRANTS};
use crate::matter_tree::{CellLocalisable, CellPart};
use crate::space_entity::SpaceEntity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceTree {
    pub scale: u32,
    pub sub_trees: [Option<Box<Self>>; NB_QUADRANTS],
    pub entities: Vec<Box<SpaceEntity>>,

    scale_factor: i64,
}

enum QuadrantMoveOperation {
    ToSubCell { quadrant: Quadrant },
    ToUpperCell,
}

impl SpaceTree {
    const NONE_SPACE_CELL: Option<Box<Self>> = None;

    pub fn new(scale: u32) -> Self {
        // Objects can't be bigger than 0x4FFF_FFFF_FFFF_FFFF.
        // Thus no point in having a bigger scale factor than that.
        let mut scale_factor = 0x4000_0000_0000_0000;
        if scale < 64 {
            scale_factor = 1;
            for _ in 0..scale {
                scale_factor *= 2;
            }
        }

        Self {
            scale,
            sub_trees: [Self::NONE_SPACE_CELL; NB_QUADRANTS],
            entities: vec![],
            scale_factor,
        }
    }

    fn move_entities_to_quadrant(&mut self, entity: Box<SpaceEntity>, quadrant: Quadrant) {
        let quadrant_i = quadrant as usize;
        if self.sub_trees[quadrant_i].is_none() {
            self.sub_trees[quadrant_i] = Some(Box::new(Self::new(self.scale - 1)));
        }
        self.sub_trees[quadrant_i]
            .as_mut()
            .unwrap()
            .add_entity(entity);
    }

    pub fn add_entity(&mut self, mut entity: Box<SpaceEntity>) -> Option<Box<SpaceEntity>> {
        match entity.get_containing_cell_part(self.scale_factor) {
            CellPart::Quadrant(quadrant) => {
                if self.scale > 0 {
                    entity.expand(quadrant);
                    self.move_entities_to_quadrant(entity, quadrant);
                } else {
                    self.entities.push(entity);
                }
                None
            }
            CellPart::MultiQuadrant => {
                self.entities.push(entity);
                None
            }
            CellPart::Outside => Some(entity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sub_trees.iter().all(|cell| cell.is_none()) && self.entities.is_empty()
    }

    pub fn refresh(&mut self) -> Vec<Box<SpaceEntity>> {
        let mut quitters = vec![];

        // Run each entity dynamics and catch crossing cell boundaries
        for (i, entity) in self.entities.iter_mut().enumerate() {
            // Check if entity should change cell
            match entity.get_containing_cell_part(self.scale_factor) {
                CellPart::MultiQuadrant => (),
                CellPart::Outside => quitters.push((i, QuadrantMoveOperation::ToUpperCell)),
                CellPart::Quadrant(quadrant) => {
                    if self.scale > 0 {
                        quitters.push((i, QuadrantMoveOperation::ToSubCell { quadrant }))
                    }
                }
            }
        }

        // Apply entity cell boundary crossing
        let mut outsiders = vec![];
        for (i, quitter) in quitters.into_iter().rev() {
            let entity = self.entities.remove(i);
            match quitter {
                QuadrantMoveOperation::ToUpperCell => outsiders.push(entity),
                QuadrantMoveOperation::ToSubCell { quadrant } => {
                    self.move_entities_to_quadrant(entity, quadrant)
                }
            }
        }

        // Run quadrants
        let mut to_move = vec![];
        {
            let Self {
                sub_trees,
                entities,
                scale_factor,
                ..
            } = self;
            for (i, quad) in sub_trees.iter_mut().enumerate() {
                if let Some(quad) = quad {
                    for mut entity in quad.refresh().into_iter() {
                        match entity.get_containing_cell_part(*scale_factor) {
                            CellPart::MultiQuadrant => {
                                entities.push(entity);
                            }
                            CellPart::Outside => {
                                entity.shrink(num::FromPrimitive::from_usize(i).unwrap());
                                outsiders.push(entity);
                            }
                            CellPart::Quadrant(quadrant) => {
                                if self.scale > 0 {
                                    to_move.push((entity, quadrant));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (mut entity, quadrant) in to_move.into_iter() {
            entity.expand(quadrant);
            self.move_entities_to_quadrant(entity, quadrant);
        }

        // Clean empty quadrants
        for i in 0..NB_QUADRANTS {
            let mut need_emptying = false;
            if let Some(quad) = self.sub_trees[i].as_ref() {
                if quad.is_empty() {
                    need_emptying = true;
                }
            }
            if need_emptying {
                self.sub_trees[i] = None;
            }
        }

        outsiders
    }
}
