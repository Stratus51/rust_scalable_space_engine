use crate::{
    geometry::{Mat3, Vec3, NB_QUADRANTS},
    matter_tree::MatterTree,
};

pub const CHUNK_SIZE: usize = 32;
pub const NB_VOXELS_PER_CHUNK: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VoxelType {
    Empty,
    Rock,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VoxelTree {
    Parent(VoxelTreeParent),
    Chunk(Box<[VoxelType; NB_VOXELS_PER_CHUNK]>),
}

impl VoxelTree {
    pub fn new_chunk() -> Self {
        Self::Chunk(Box::new([VoxelType::Empty; NB_VOXELS_PER_CHUNK]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoxelTreeParent {
    pub scale: u32,
    pub sub_cells: [Option<Box<Self>>; NB_QUADRANTS],
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoxelGridSpace {
    pub voxels: VoxelTree,
    pub local_space: MatterTree,
    pub orientation: Mat3,
}

impl VoxelGridSpace {
    fn new(pos: Vec3) -> Self {
        Self {
            voxels: VoxelTree::new_chunk(),
            local_space: MatterTree::new(),
            orientation: Mat3::IDENTITY,
        }
    }
}
