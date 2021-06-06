use crate::geometry::{Mat3, Vec3};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpaceEntityData {
    // Localisation
    pub pos: Vec3,
    pub size: Vec3,
    pub orientation: Mat3,

    // Dynamics
    pub speed: Vec3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpaceEntity {
    // Localisation
    pub pos: Vec3,
    pub size: Vec3,
    pub orientation: Mat3,

    // Optimization related
    pub pos_max: Vec3,
}

pub trait WithSpaceEntity {
    fn space_entity(&self) -> &SpaceEntity;
    fn space_entity_mut(&mut self) -> &mut SpaceEntity;
}
