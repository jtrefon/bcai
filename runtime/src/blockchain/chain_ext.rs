use super::{chain::Blockchain, block::Block};

/// Helper methods for `Blockchain` extracted to keep files small.
impl Blockchain {
    /// Returns a reference to the last block in the chain, if any.
    pub fn get_last_block(&self) -> Option<&Block> {
        self.blocks.last()
    }
} 