use std::{mem::MaybeUninit, ops::Range};

use bevy_ecs::world::World as BevyECS;
use bytemuck::{Pod, Zeroable};

use crate::math::{Point, AABB};

use super::chunk::ArrayChunk;

pub type BlockPos = i64;
pub type ChunkPos = i32;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_DISTANCE: usize = 10;

#[derive(Debug, Clone, Copy, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct BlockCoord {
    x: BlockPos,
    y: BlockPos,
    z: BlockPos,
}

impl Point<BlockPos, 3> for BlockCoord {
    fn to_array(&self) -> [BlockPos; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Debug, Clone, Copy, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct ChunkCoord {
    x: ChunkPos,
    y: ChunkPos,
    z: ChunkPos,
}

impl Point<ChunkPos, 3> for ChunkCoord {
    fn to_array(&self) -> [ChunkPos; 3] {
        [self.x, self.y, self.z]
    }
}

pub struct Terrain {
    requested_chunks: Vec<ChunkCoord>,
    loaded_chunks: ArrayChunk,
}

impl Terrain {
    pub fn new() -> Self {
        Terrain {
            requested_chunks: vec![],
            loaded_chunks: ArrayChunk::default(),
        }
    }

    pub fn slice<'a>(&'a self, selection: AABB<BlockPos, BlockCoord>) -> TerrainSlice<'a> {
        TerrainSlice {
            of: self,
            selection,
        }
    }
}

pub struct TerrainSlice<'a> {
    of: &'a Terrain,
    selection: AABB<BlockPos, BlockCoord>,
}

pub struct World {
    ecs: BevyECS,
    terrain: Terrain,
}

impl World {
    fn new() {
        let ecs = BevyECS::default();
    }
}
