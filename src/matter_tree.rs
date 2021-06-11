use crate::entity::Entity;
use crate::geometry::{Cube, FineDirection, Quadrant, Vec3, NB_QUADRANTS};

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
    pub const MIN_SIZE: i64 = 1 << Self::MIN_SIZE_POW;
    const MAX_SCALE: u32 = 64 // Max
        - 1 // Remove sign
        - Self::MIN_SIZE_POW as u32 // Remove scales taken up by min size cells
        - 1; // Margin
    pub const MAX_SIZE: i64 = 1 << (Self::MIN_SIZE_POW + Self::MAX_SCALE as i64);
    const NONE_SPACE_CELL: Option<Box<Self>> = None;

    pub fn new() -> Self {
        Self::new_tree(
            Self::MAX_SCALE,
            Cube {
                origin: Vec3 {
                    x: -Self::MAX_SIZE / 2,
                    y: -Self::MAX_SIZE / 2,
                    z: -Self::MAX_SIZE / 2,
                },
                size: Self::MAX_SIZE,
            },
        )
    }

    fn new_tree(scale: u32, area: Cube) -> Self {
        Self {
            scale,
            sub_trees: [Self::NONE_SPACE_CELL; NB_QUADRANTS],
            entities: vec![],
            area,
        }
    }

    fn new_sub_tree(&self, quadrant: Quadrant) -> Self {
        let origin = self.area.origin;
        let size = self.area.size;
        Self::new_tree(
            self.scale - 1,
            Cube {
                origin: Vec3 {
                    x: origin.x + quadrant.x_p() as i64 * size,
                    y: origin.y + quadrant.y_p() as i64 * size,
                    z: origin.z + quadrant.z_p() as i64 * size,
                },
                size,
            },
        )
    }

    fn move_entities_to_quadrant(&mut self, entities: Vec<Box<Entity>>, quadrant: Quadrant) {
        let quadrant_i = quadrant as usize;
        if self.sub_trees[quadrant_i].is_none() {
            self.sub_trees[quadrant_i] = Some(Box::new(self.new_sub_tree(quadrant)));
        }
        // TODO Cleaner way to do this in rust?
        self.sub_trees[quadrant_i]
            .as_mut()
            .unwrap()
            .add_entities(entities);
    }

    fn center(&self) -> Vec3 {
        let half = self.area.size / 2;
        self.area.origin.add(&Vec3 {
            x: half,
            y: half,
            z: half,
        })
    }

    pub fn add_entities(&mut self, mut entities: Vec<Box<Entity>>) {
        // TODO Is that the right condition to decide whether to split the space?
        if self.entities.len() + entities.len() == 1 || self.scale == 0 {
            self.entities.extend(entities);
        } else {
            let mut per_quadrant = vec![vec![]; NB_QUADRANTS];
            for entity in entities.into_iter() {
                let relative_sphere = entity.bounding_sphere.sub_to_center(&self.center());
                let quadrant = Quadrant::from_pos(&relative_sphere.center);
                if relative_sphere.is_inside_quadrant(self.area.size, quadrant as usize) {
                    per_quadrant[quadrant as usize].push(entity);
                } else {
                    self.entities.push(entity);
                }
            }

            for (i, entities) in per_quadrant.into_iter().enumerate() {
                self.move_entities_to_quadrant(
                    entities,
                    num::FromPrimitive::from_usize(i).unwrap(),
                );
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
        let mut insiders = vec![vec![]; NB_QUADRANTS];
        let mut outsiders = vec![];
        for (i, quitter) in quitters.into_iter().rev() {
            let entity = self.entities.remove(i);
            match quitter {
                QuadrantMoveOperation::ToUpperCell => outsiders.push(entity),
                QuadrantMoveOperation::ToSubCell { quadrant } => {
                    insiders[quadrant as usize].push(entity);
                }
            }
        }

        // Run quadrants
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
                                    insiders[quadrant as usize].push(entity);
                                }
                            }
                        }
                    }
                }
            }
        }

        for (i, entities) in insiders.into_iter().enumerate() {
            self.move_entities_to_quadrant(entities, num::FromPrimitive::from_usize(i).unwrap());
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

    pub fn get_entities_touching_outside(&mut self) -> Vec<(&mut Box<Entity>, Vec<FineDirection>)> {
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
