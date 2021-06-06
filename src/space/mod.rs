use crate::geometry::{Mat3, Quadrant, Vec3, NB_QUADRANTS};
use std::collections::HashSet;

pub mod entity;

pub const SPACE_CELL_SIZE: u32 = 1024 * 1024;
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
    universe: Box<SpaceCell>,
}

const INVALID_ID: u64 = 0;

#[derive(Debug, PartialEq, Eq)]
struct SpaceCell {
    // Structure related
    remaining_sub_divisions: u32,

    // Data
    sub_cells: [Option<Box<SpaceCell>>; NB_QUADRANTS],
    entities: Vec<SpaceEntity>,

    // Optimization related
    contained_mass: u64, // units x8 compared to parent
}

enum QuadrantMoveOperation {
    ToSubCell { quadrant: Quadrant },
    ToUpperCell,
}

const NONE_SPACE_CELL: Option<Box<SpaceCell>> = None;
impl SpaceCell {
    pub fn new(remaining_sub_divisions: u32) -> Self {
        Self {
            remaining_sub_divisions,

            sub_cells: [NONE_SPACE_CELL; NB_QUADRANTS],
            entities: vec![],
            contained_mass: 0,
        }
    }

    fn move_entity_to_quadrant(&mut self, mut entity: SpaceEntity, quadrant: Quadrant) {
        let quadrant_i = quadrant as usize;
        if self.sub_cells[quadrant_i].is_none() {
            self.sub_cells[quadrant_i] =
                Some(Box::new(SpaceCell::new(self.remaining_sub_divisions - 1)));
        }
        entity.expand(quadrant);
        self.sub_cells[quadrant_i]
            .as_mut()
            .unwrap()
            .add_entity(entity);
    }

    pub fn add_entity(&mut self, entity: SpaceEntity) {
        if self.remaining_sub_divisions > 0 {
            if let Some(quadrant) = entity.get_quadrant() {
                self.move_entity_to_quadrant(entity, quadrant);
                return;
            }
        }
        self.entities.push(entity);
        self.contained_mass += entity.caracteristics.full_mass.mass;
    }

    pub fn is_empty(&self) -> bool {
        self.contained_mass == 0
    }

    pub fn run_movements(&mut self, tick_size: i64) -> Vec<SpaceEntity> {
        let mut quitters = vec![];

        // Run each entity dynamics and catch crossing cell boundaries
        for (i, entity) in self.entities.iter_mut().enumerate() {
            // Reset entity state
            entity.turn_reset();

            // Apply external effects
            // TODO Gravity

            // Dynamics calculations
            entity.run_movements(tick_size);

            // Check if entity should change cell
            if !entity.is_inside_centered_cube(SPACE_CELL_SIZE as i64) {
                quitters.push((i, QuadrantMoveOperation::ToUpperCell));
            } else if self.remaining_sub_divisions > 0 {
                if let Some(quadrant) = entity.get_quadrant() {
                    quitters.push((i, QuadrantMoveOperation::ToSubCell { quadrant }))
                }
            }
        }

        // Apply entity cell boundary crossing
        let mut outsiders = vec![];
        for (i, quitter) in quitters.into_iter().rev() {
            let entity = self.entities.remove(i);
            self.contained_mass -= entity.caracteristics.full_mass.mass;
            match quitter {
                QuadrantMoveOperation::ToUpperCell => outsiders.push(entity),
                QuadrantMoveOperation::ToSubCell { quadrant } => {
                    self.move_entity_to_quadrant(entity, quadrant)
                }
            }
        }

        // Run quadrant
        let mut to_move = vec![];
        {
            let Self {
                sub_cells,
                entities,
                contained_mass,
                ..
            } = self;
            for (i, quad) in sub_cells.iter_mut().enumerate() {
                if let Some(quad) = quad {
                    for mut entity in quad.run_movements(tick_size).into_iter() {
                        entity.shrink(num::FromPrimitive::from_usize(i).unwrap());
                        if !entity.is_inside_centered_cube(SPACE_CELL_SIZE as i64) {
                            outsiders.push(entity);
                        } else if let Some(quadrant) = entity.get_quadrant() {
                            to_move.push((entity, quadrant));
                        } else {
                            entities.push(entity);
                            *contained_mass += entity.caracteristics.full_mass.mass;
                        }
                    }
                }
            }
        }
        for (entity, quadrant) in to_move.into_iter() {
            self.move_entity_to_quadrant(entity, quadrant);
        }

        // Clean empty quadrants
        for i in 0..NB_QUADRANTS {
            let mut need_emptying = false;
            if let Some(quad) = self.sub_cells[i].as_ref() {
                if quad.is_empty() {
                    need_emptying = true;
                }
            }
            if need_emptying {
                self.sub_cells[i] = None;
            }
        }

        outsiders
    }

    pub fn run_collisions(&mut self) {}
}

type SpaceEntityId = u64;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PonctualMass {
    pub pos: Vec3,
    pub mass: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpaceEntityCaracteristics {
    pub size: Vec3,
    pub full_mass: PonctualMass,
    pub split_mass: [PonctualMass; NB_QUADRANTS],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpaceEntityData {
    pub id: SpaceEntityId,

    // Localisation
    pub pos: Vec3,         // x=0, y=0, z=0 corner. Units are SpaceCell relative.
    pub orientation: Mat3, // 1000 normalized

    // Dynamics
    pub speed: Vec3, // cm/tick

    // Caracteristics
    pub caracteristics: SpaceEntityCaracteristics,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpaceEntity {
    id: SpaceEntityId,

    // Localisation
    pos: Vec3,         // x=0, y=0, z=0 corner. Units are SpaceCell relative.
    orientation: Mat3, // 1000 normalized

    // Dynamics
    speed: Vec3,        // cm/tick
    acceleration: Vec3, // cm/tick^2

    // Caracteristics
    caracteristics: SpaceEntityCaracteristics,

    // Optimization related
    pos_max: Vec3,
    gravity_radius: u64,
}

impl SpaceEntity {
    pub fn new(data: SpaceEntityData) -> Self {
        let mut ret = Self {
            id: data.id,

            // Localisation
            pos: data.pos,
            orientation: data.orientation,

            // Dynamics
            speed: data.speed,
            acceleration: Vec3::zero(),

            // Caracteristics
            caracteristics: data.caracteristics,

            // Optimization related
            pos_max: Vec3::zero(),
            gravity_radius: 0, // TODO
        };
        ret.refresh_max_pos();
        ret
    }

    pub fn is_inside_centered_cube(&self, side_length: i64) -> bool {
        self.pos.is_inside_centered_cube(side_length)
            && self.pos_max.is_inside_centered_cube(side_length)
    }

    pub fn get_quadrant(&self) -> Option<Quadrant> {
        let origin = Quadrant::from_pos(&self.pos);
        let max = Quadrant::from_pos(&self.pos_max);
        if origin == max {
            Some(origin)
        } else {
            None
        }
    }

    pub fn get_corners(&self) -> [Vec3; 8] {
        let mut ret = [Vec3::zero(); 8];
        for i in 0..7 {
            let mut shift = self.caracteristics.size;
            shift.x *= i & (1 << 2);
            shift.y *= i & (1 << 1);
            shift.z *= i & (1 << 0);
            ret[i as usize] = self.pos.add(&self.orientation.mul_vec(&shift))
        }
        ret
    }

    pub fn get_quadrants(&self) -> Vec<Quadrant> {
        let mut ret = HashSet::new();
        for corner in self.get_corners().into_iter() {
            ret.insert(corner.get_quadrant());
        }
        ret.into_iter().collect()
    }

    pub fn refresh_max_pos(&mut self) {
        self.pos_max = self
            .pos
            .add(&self.orientation.mul_vec(&self.caracteristics.size));
    }

    pub fn geometric_center(&self) -> Vec3 {
        self.pos.add(
            &self
                .orientation
                .mul_vec(&self.caracteristics.size)
                .div_scalar(2),
        )
    }

    pub fn expand(&mut self, quadrant: Quadrant) {
        let quadrant = quadrant as usize;
        let shift = SPACE_CELL_SIZE as i64 / 4;
        if quadrant & (1 << 2) != 0 {
            self.pos.x -= shift;
        } else {
            self.pos.x += shift;
        }
        if quadrant & (1 << 1) != 0 {
            self.pos.y -= shift;
        } else {
            self.pos.y += shift;
        }
        if quadrant != 0 {
            self.pos.z -= shift;
        } else {
            self.pos.z += shift;
        }
        self.pos = self.pos.mul_scalar(2);
        self.caracteristics.size = self.pos.mul_scalar(2);
        self.caracteristics.full_mass.pos = self.pos.mul_scalar(2);
        self.caracteristics.full_mass.mass *= 8;
        for dot in self.caracteristics.split_mass.iter_mut() {
            dot.pos = self.pos.mul_scalar(2);
            dot.mass *= 8;
        }
        self.refresh_max_pos();
    }

    pub fn shrink(&mut self, quadrant: Quadrant) {
        let quadrant = quadrant as usize;
        self.pos = self.pos.div_scalar(2);
        let shift = SPACE_CELL_SIZE as i64 / 2;
        if quadrant & (1 << 2) != 0 {
            self.pos.x += shift;
        } else {
            self.pos.x -= shift;
        }
        if quadrant & (1 << 1) != 0 {
            self.pos.y += shift;
        } else {
            self.pos.y -= shift;
        }
        if quadrant != 0 {
            self.pos.z += shift;
        } else {
            self.pos.z -= shift;
        }
        self.caracteristics.size = self.pos.div_scalar(2);
        self.caracteristics.full_mass.pos = self.pos.div_scalar(2);
        self.caracteristics.full_mass.mass /= 8;
        for dot in self.caracteristics.split_mass.iter_mut() {
            dot.pos = self.pos.div_scalar(2);
            dot.mass /= 8;
        }
        self.refresh_max_pos();
    }

    pub fn turn_reset(&mut self) {
        self.acceleration = Vec3::zero();
    }

    pub fn run_movements(&mut self, tick_size: i64) {
        self.pos = self
            .pos
            .add(&self.speed.mul_scalar(tick_size).div_scalar(TICK_DIV));

        self.speed = self
            .speed
            .add(&self.acceleration.mul_scalar(tick_size).div_scalar(TICK_DIV));

        self.refresh_max_pos();
        println!("Entity: pos = {:?} | speed = {:?}", self.pos, self.speed);
    }
}

impl Space {
    pub fn new(conf: SpaceConfiguration) -> Self {
        Self {
            conf,
            universe: Box::new(SpaceCell::new(0)),
        }
    }

    pub fn insert_entity(&mut self, mut entity: SpaceEntity) {
        // Expand universe until entity origin is in it
        while !entity.pos.is_inside_centered_cube(SPACE_CELL_SIZE as i64) {
            // Enlarge universe
            let quadrant = entity.pos.get_quadrant().invert();
            let new_universe = Box::new(SpaceCell::new(self.universe.remaining_sub_divisions + 1));
            let sub_cell = std::mem::replace(&mut self.universe, new_universe);
            self.universe.sub_cells[quadrant as usize] = Some(sub_cell);

            // Adjust entity
            entity.shrink(quadrant);
        }

        // Expand universe until entity origin is in it
        while !entity
            .pos_max
            .is_inside_centered_cube(SPACE_CELL_SIZE as i64)
        {
            // Enlarge universe
            let quadrant = entity.pos_max.get_quadrant().invert();
            let new_universe = Box::new(SpaceCell::new(self.universe.remaining_sub_divisions + 1));
            let sub_cell = std::mem::replace(&mut self.universe, new_universe);
            self.universe.sub_cells[quadrant as usize] = Some(sub_cell);

            // Adjust entity
            entity.shrink(quadrant);
        }
        self.universe.add_entity(entity);
    }

    pub fn run(&mut self) {
        let outsiders = self.universe.run_movements(self.conf.tick_size);
        for entity in outsiders.into_iter() {
            self.insert_entity(entity);
        }

        self.universe.run_collisions();
    }
}
