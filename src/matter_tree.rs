use crate::{
    entity::{Entity, EntityData},
    geometry::{Cube, FineDirection, Quadrant, Sphere, Vec3, NB_QUADRANTS},
    voxel_grid::VoxelGridSpace,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CellPart {
    CenterOutside,
    PartlyOutside,
    MultiQuadrant,
    Quadrant(Quadrant),
}

type Entities = Vec<Box<Entity>>;

#[derive(Debug, Clone, PartialEq)]
pub struct MatterTree {
    pub scale: u32,
    pub sub_trees: [Option<Box<Self>>; NB_QUADRANTS],
    pub entities: Entities,

    pub area: Cube,
}

enum QuadrantMoveOperation {
    ToSubCell { quadrant: Quadrant },
    ToUpperCell,
}

impl MatterTree {
    const MIN_SIZE_POW: i64 = 5;
    pub const MIN_SIZE: i64 = 1 << Self::MIN_SIZE_POW;
    const MAX_SCALE: u32 = 64 // Max
        - 1 // Remove sign
        - Self::MIN_SIZE_POW as u32 // Remove scales taken up by min size cells
        - 1 // Margin
        - 50; // Manual testing
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
        let size = self.area.size / 2;
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

    fn move_entities_to_quadrant(&mut self, entities: Entities, quadrant: Quadrant) {
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

    pub fn add_entities(&mut self, entities: Entities) {
        // TODO Is that the right condition to decide whether to split the space?
        if self.entities.len() + entities.len() <= 1 || self.scale == 0 {
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
                if !entities.is_empty() {
                    self.move_entities_to_quadrant(
                        entities,
                        num::FromPrimitive::from_usize(i).unwrap(),
                    );
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sub_trees.iter().all(|cell| cell.is_none()) && self.entities.is_empty()
    }

    pub fn refresh(&mut self) -> Entities {
        let mut quitters = vec![];

        // Run each entity dynamics and catch crossing cell boundaries
        for (i, entity) in self.entities.iter_mut().enumerate() {
            // Check if entity should change cell
            let cell_part = entity.get_containing_cell_part(&self.area);
            match cell_part {
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
            for quad in sub_trees.iter_mut() {
                if let Some(quad) = quad {
                    for entity in quad.refresh().into_iter() {
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

        if self.entities.len() + insiders.iter().map(|v| v.len()).sum::<usize>() == 1 {
            for insider in insiders.into_iter() {
                self.entities.extend(insider);
            }
        } else {
            for (i, entities) in insiders.into_iter().enumerate() {
                if !entities.is_empty() {
                    self.move_entities_to_quadrant(
                        entities,
                        num::FromPrimitive::from_usize(i).unwrap(),
                    );
                }
            }
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
            let source = source.last_mut().unwrap();
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
        for (i, sub_tree) in sub_trees.iter_mut().enumerate() {
            if let Some(quad) = sub_tree {
                let mut relevant_entities: Vec<_> = entities
                    .iter_mut()
                    .enumerate()
                    .filter(|(j, _)| entity_quadrant[*j].contains(&(i as u8)))
                    .map(|(_, e)| e)
                    .collect();
                quad.apply_external_collisions(&mut relevant_entities[..]);
            }
        }
    }

    pub fn get_entities_touching_outside(&mut self) -> Vec<(&mut Box<Entity>, Vec<FineDirection>)> {
        let area = &self.area;
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

    pub fn apply_external_collisions(&mut self, outsiders: &mut [&mut Box<Entity>]) {
        for a in self.entities.iter_mut() {
            for b in outsiders.iter_mut() {
                a.apply_collision(b);
            }
        }
    }

    pub fn run_actions(&mut self) {
        for i in 0..self.entities.len() {
            let drop_rock = match &self.entities[i].entity {
                EntityData::Player(player) => player.borrow().drop_block,
                _ => false,
            };
            if drop_rock {
                let rock = {
                    let player = &self.entities[i];
                    let grid = VoxelGridSpace::new();
                    let mut entity = Entity::new(
                        Sphere {
                            center: player.bounding_sphere.center.sub(&player.speed),
                            radius: 1,
                        },
                        EntityData::Voxels(Box::new(grid)),
                    );
                    entity.speed = player.speed;
                    entity
                };
                self.entities.push(Box::new(rock));
            }
        }

        for sub_tree in self.sub_trees.iter_mut() {
            if let Some(tree) = sub_tree {
                tree.run_actions();
            }
        }
    }

    pub fn run_movements(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.run_movement();
        }
        for sub_tree in self.sub_trees.iter_mut() {
            if let Some(tree) = sub_tree {
                tree.run_movements();
            }
        }
    }

    pub fn nb_nodes(&self) -> usize {
        self.sub_trees
            .iter()
            .map(|opt| match opt {
                Some(tree) => tree.nb_nodes(),
                None => 0,
            })
            .sum()
    }

    pub fn nb_entities(&self) -> usize {
        self.entities.len()
            + self
                .sub_trees
                .iter()
                .map(|opt| match opt {
                    Some(tree) => tree.nb_entities(),
                    None => 0,
                })
                .sum::<usize>()
    }
}
