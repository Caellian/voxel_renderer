use std::default;

use crate::block::BlockID;

use super::world::CHUNK_SIZE;

#[derive(Debug, Clone)]
pub struct ArrayChunk {
    blocks: [[[BlockID; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Default for ArrayChunk {
    fn default() -> Self {
        ArrayChunk {
            blocks: [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}
