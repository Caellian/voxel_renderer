use crate::{
    block::BlockID,
    content::world::{BlockCoord, BlockPos, TerrainSlice},
    math::AABB,
};

pub enum Change {
    Clear,
    Set(BlockID),
}

pub struct GenerationResult {
    on_load: Vec<Change>,
}

pub trait Generator {}

pub fn pass<'w>(world_slice: TerrainSlice<'w>, working_area: AABB<BlockPos, BlockCoord>) {}
