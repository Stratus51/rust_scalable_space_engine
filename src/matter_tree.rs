use crate::entity::Entity;
use crate::geometry::{Cube, FineDirection, Quadrant, Vec3, NB_QUADRANTS};
use itertools::Itertools;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CellPart {
    CenterOutside,
    PartlyOutside,
    MultiQuadrant,
    Quadrant(Quadrant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatterTree {
    pub scale: u32,
    pub sub_trees: [Option<Box<Self>>; NB_QUADRANTS],
    pub entities: Vec<Box<Entity>>,

    pub area: Cube,
}

enum QuadrantMoveOperation {
    ToSubCell { quadrant: Quadrant },
    ToUpperCell,
}

impl MatterTree {
    const MIN_SIZE_POW: i64 = 20;
    const MIN_SIZE: i64 = 1 << Self::MIN_SIZE_POW;
    const MAX_SCALE: u32 = 64 // Max
        - 1 // Remove sign
        - Self::MIN_SIZE_POW as u32 // Remove scales taken up by min size cells
        - 1; // Margin
    const NONE_SPACE_CELL: Option<Box<Self>> = None;

    pub fn new(area: Cube) -> Self {
        Self {
            scale: Self::MAX_SCALE,
            sub_trees: [Self::NONE_SPACE_CELL; NB_QUADRANTS],
            entities: vec![],
            area,
        }
    }

    fn move_entities_to_quadrant(&mut self, entity: Box<Entity>, quadrant: Quadrant) {
        let quadrant_i = quadrant as usize;
        if self.sub_trees[quadrant_i].is_none() {
            let parent = &self.area;
            let origin = parent.origin;
            let size = parent.size / 2;
            self.sub_trees[quadrant_i] = Some(Box::new(Self::new(Cube {
                origin: Vec3 {
                    x: origin.x + quadrant.x_p() as i64 * size,
                    y: origin.y + quadrant.y_p() as i64 * size,
                    z: origin.z + quadrant.z_p() as i64 * size,
                },
                size,
            })));
        }
        self.sub_trees[quadrant_i]
            .as_mut()
            .unwrap()
            .add_entity(entity);
    }

    pub fn add_entity(&mut self, mut entity: Box<Entity>) -> Option<Box<Entity>> {
        // TODO Is that the right condition to decide whether to split the space?
        if self.entities.is_empty() {
            self.entities.push(entity);
            None
        } else {
            match entity.get_containing_cell_part(&self.area) {
                CellPart::Quadrant(quadrant) => {
                    if self.scale > 0 {
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
                CellPart::CenterOutside => Some(entity),
                CellPart::PartlyOutside => {
                    if self.scale == Self::MAX_SCALE {
                        self.entities.push(entity);
                        None
                    } else {
                        Some(entity)
                    }
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sub_trees.iter().all(|cell| cell.is_none()) && self.entities.is_empty()
    }

    pub fn refresh(&mut self) -> Vec<Box<Entity>> {
        let mut quitters = vec![];

        // Run each entity dynamics and catch crossing cell boundaries
        for (i, entity) in self.entities.iter_mut().enumerate() {
            // Check if entity should change cell
            match entity.get_containing_cell_part(&self.area) {
                CellPart::MultiQuadrant => (),
                CellPart::PartlyOutside => {
                    if self.scale < Self::MAX_SCALE {
                        quitters.push((i, QuadrantMoveOperation::ToUpperCell))
                    }
                }
                CellPart::CenterOutside => quitters.push((i, QuadrantMoveOperation::ToUpperCell)),
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
                area,
                ..
            } = self;
            for (i, quad) in sub_trees.iter_mut().enumerate() {
                if let Some(quad) = quad {
                    for mut entity in quad.refresh().into_iter() {
                        match entity.get_containing_cell_part(area) {
                            CellPart::MultiQuadrant => {
                                entities.push(entity);
                            }
                            CellPart::PartlyOutside => {
                                if self.scale < Self::MAX_SCALE {
                                    outsiders.push(entity);
                                }
                            }
                            CellPart::CenterOutside => {
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

    pub fn apply_neighbourhood_collisions(&mut self) {
        // Apply collisions to entities of this node
        let mut entity_quadrant = vec![];
        let area = &self.area;
        for i in 0..self.entities.len() {
            let (source, remainder) = self.entities.split_at_mut(i + 1);
            let mut source = source.last_mut().unwrap();
            for e in remainder.iter_mut() {
                source.apply_collision(e);
            }
            entity_quadrant.push(source.get_collisioned_quadrants(area));
        }

        // Apply collisions to all sub_tree entities
        let Self {
            sub_trees,
            entities,
            ..
        } = self;
        for i in 0..NB_QUADRANTS {
            if let Some(quad) = sub_trees[i].as_mut() {
                let relevant_entities: Vec<_> = entities
                    .iter_mut()
                    .enumerate()
                    .filter(|(j, _)| entity_quadrant[*j].contains(&(i as u8)))
                    .map(|(_, e)| e)
                    .collect();
                quad.apply_external_collisions(&relevant_entities[..]);
            }
        }
    }

    pub fn get_outsiders(&mut self) -> Vec<(&mut Box<Entity>, Vec<FineDirection>)> {
        let Self { entities, area, .. } = self;
        self.entities
            .iter_mut()
            .filter_map(|e| {
                let directions = e.get_touched_external_cells(area);
                if directions.is_empty() {
                    None
                } else {
                    Some((e, directions))
                }
            })
            .collect()
    }

    pub fn apply_external_collisions(&mut self, outsiders: &[&mut Box<Entity>]) {
        for a in self.entities.iter_mut() {
            for b in outsiders.iter_mut() {
                a.apply_collision(b);
            }
        }
    }
}
