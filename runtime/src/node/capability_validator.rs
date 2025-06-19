use super::{
    node::UnifiedNode,
    types::NodeCapability,
};

impl UnifiedNode {
    /// Checks if the node's capabilities meet the job's requirements.
    pub(super) fn meets_capability_requirements(&self, required: &NodeCapability) -> bool {
        self.capability.cpus >= required.cpus
            && self.capability.gpus >= required.gpus
            && self.capability.gpu_memory_gb >= required.gpu_memory_gb
            && self.capability.available_stake >= required.available_stake
    }
} 