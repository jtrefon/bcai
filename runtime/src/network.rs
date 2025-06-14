//! Network integration layer bridging P2P communication with unified node architecture
//!
//! This module provides the glue between libp2p networking and BCAI's distributed training system.

use crate::node::{DistributedJob, NodeCapability, TrainingResult, UnifiedNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Network message types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkMessage {
    Ping,
    Pong,
    Data(Vec<u8>),
    Request(String),
    Response(String),
}

/// Network coordination errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Network partition detected")]
    NetworkPartition,
    #[error("Message validation failed")]
    InvalidMessage,
    #[error("Consensus failure")]
    ConsensusFailed,
}

/// Network coordinator for managing network operations
#[derive(Debug, Clone, Default)]
pub struct NetworkCoordinator {
    node_id: String,
    peers: Vec<String>,
}

impl NetworkCoordinator {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            peers: Vec::new(),
        }
    }
    
    pub fn add_peer(&mut self, peer_id: String) {
        self.peers.push(peer_id);
    }
    
    pub fn broadcast(&self, _message: NetworkMessage) -> Result<(), String> {
        // Stub implementation
        Ok(())
    }
    
    pub fn send_to_peer(&self, _peer_id: &str, _message: NetworkMessage) -> Result<(), String> {
        // Stub implementation
        Ok(())
    }
    
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    
    pub fn peers(&self) -> &[String] {
        &self.peers
    }
}

/// Network coordinator managing distributed operations
pub struct NetworkCoordinator {
    local_node: UnifiedNode,
    peer_capabilities: HashMap<String, NodeCapability>,
    global_job_registry: HashMap<u64, DistributedJob>,
    pending_evaluations: HashMap<u64, Vec<(String, bool)>>, // job_id -> (evaluator_id, is_valid)
    network_block_height: u64,
}

impl NetworkCoordinator {
    /// Create a new network coordinator
    pub fn new(local_node: UnifiedNode) -> Self {
        Self {
            local_node,
            peer_capabilities: HashMap::new(),
            global_job_registry: HashMap::new(),
            pending_evaluations: HashMap::new(),
            network_block_height: 1,
        }
    }

    /// Handle incoming network message
    pub fn handle_message(
        &mut self,
        message: NetworkMessage,
        _sender_id: &str,
    ) -> Result<Vec<NetworkMessage>, NetworkError> {
        let mut responses = Vec::new();

        match message {
            NetworkMessage::CapabilityAnnouncement { node_id, capability } => {
                self.peer_capabilities.insert(node_id, capability);
            }

            NetworkMessage::JobPosted { job, poster_id: _ } => {
                // Add to global registry
                self.global_job_registry.insert(job.id, job.clone());

                // Check if local node can handle this job
                if self.can_handle_job(&job) {
                    responses.push(NetworkMessage::JobVolunteer {
                        job_id: job.id,
                        node_id: self.local_node.node_id.clone(),
                        capability: self.local_node.capability.clone(),
                    });
                }
            }

            NetworkMessage::JobVolunteer { job_id, node_id, capability: _ } => {
                if let Some(job) = self.global_job_registry.get_mut(&job_id) {
                    if job.assigned_workers.len() < 3 && !job.assigned_workers.contains(&node_id) {
                        job.assigned_workers.push(node_id);

                        // If we have enough workers, start training
                        if job.assigned_workers.len() >= 3 {
                            job.status = crate::node::JobStatus::WorkersAssigned;
                        }
                    }
                }
            }

            NetworkMessage::TrainingResultSubmission { result, submitter_id: _ } => {
                // Store result for evaluation
                if let Ok(is_valid) =
                    self.local_node.evaluate_training_result(result.job_id, &result)
                {
                    responses.push(NetworkMessage::TrainingEvaluation {
                        job_id: result.job_id,
                        result_hash: result.model_hash.clone(),
                        is_valid,
                        evaluator_id: self.local_node.node_id.clone(),
                    });
                }
            }

            NetworkMessage::TrainingEvaluation {
                job_id,
                result_hash: _,
                is_valid,
                evaluator_id,
            } => {
                // Collect evaluations
                let evaluations = self.pending_evaluations.entry(job_id).or_default();
                evaluations.push((evaluator_id, is_valid));

                // Check if we have consensus (majority agreement)
                if evaluations.len() >= 2 {
                    let valid_count = evaluations.iter().filter(|(_, valid)| *valid).count();
                    let total_count = evaluations.len();

                    if valid_count * 2 > total_count {
                        // Majority agrees it's valid - complete the job
                        if let Ok(()) = self.local_node.complete_distributed_job(job_id) {
                            if self.global_job_registry.contains_key(&job_id) {
                                responses.push(NetworkMessage::JobCompleted {
                                    job_id,
                                    final_model_hash: format!("final_model_{}", job_id),
                                });
                            }
                        }
                        self.pending_evaluations.remove(&job_id);
                    }
                }
            }

            NetworkMessage::JobCompleted { job_id, final_model_hash: _ } => {
                if let Some(job) = self.global_job_registry.get_mut(&job_id) {
                    job.status = crate::node::JobStatus::Completed;
                }
            }

            NetworkMessage::StateSync { requesting_node: _, last_known_block } => {
                let jobs_to_sync: Vec<DistributedJob> = self
                    .global_job_registry
                    .values()
                    .filter(|job| job.created_block > last_known_block)
                    .cloned()
                    .collect();

                responses.push(NetworkMessage::StateSyncResponse {
                    jobs: jobs_to_sync,
                    current_block: self.network_block_height,
                });
            }

            NetworkMessage::StateSyncResponse { jobs, current_block } => {
                // Update network state
                for job in jobs {
                    self.global_job_registry.insert(job.id, job);
                }
                self.network_block_height = current_block;
            }
        }

        Ok(responses)
    }

    /// Check if local node can handle a job
    fn can_handle_job(&self, job: &DistributedJob) -> bool {
        self.local_node.capability.cpus >= job.required_capability.cpus
            && self.local_node.capability.gpus >= job.required_capability.gpus
            && self.local_node.capability.gpu_memory_gb >= job.required_capability.gpu_memory_gb
            && self.local_node.staked() >= job.required_capability.available_stake
    }

    /// Post a job to the network
    pub fn post_job_to_network(
        &mut self,
        description: String,
        reward: u64,
        required_capability: NodeCapability,
        data_hash: String,
        model_spec: String,
        deadline_blocks: u64,
    ) -> Result<NetworkMessage, crate::node::NodeError> {
        let job_id = self.local_node.post_distributed_job(
            description,
            reward,
            required_capability,
            data_hash,
            model_spec,
            deadline_blocks,
        )?;

        let job = self.local_node.distributed_jobs().get(&job_id).unwrap().clone();
        self.global_job_registry.insert(job_id, job.clone());

        Ok(NetworkMessage::JobPosted { job, poster_id: self.local_node.node_id.clone() })
    }

    /// Execute training for a job and broadcast results
    pub fn execute_training_and_broadcast(
        &mut self,
        job_id: u64,
    ) -> Result<NetworkMessage, crate::node::NodeError> {
        let result = self.local_node.execute_training(job_id, 0x0000ffff)?;

        Ok(NetworkMessage::TrainingResultSubmission {
            result,
            submitter_id: self.local_node.node_id.clone(),
        })
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            connected_peers: self.peer_capabilities.len(),
            active_jobs: self
                .global_job_registry
                .values()
                .filter(|job| {
                    !matches!(
                        job.status,
                        crate::node::JobStatus::Completed | crate::node::JobStatus::Failed
                    )
                })
                .count(),
            completed_jobs: self
                .global_job_registry
                .values()
                .filter(|job| job.status == crate::node::JobStatus::Completed)
                .count(),
            network_block_height: self.network_block_height,
            local_node_stats: self.local_node.get_stats(),
        }
    }

    /// Synchronize with network state
    pub fn request_state_sync(&self) -> NetworkMessage {
        NetworkMessage::StateSync {
            requesting_node: self.local_node.node_id.clone(),
            last_known_block: self.network_block_height,
        }
    }

    /// Announce capabilities to network
    pub fn announce_capabilities(&self) -> NetworkMessage {
        NetworkMessage::CapabilityAnnouncement {
            node_id: self.local_node.node_id.clone(),
            capability: self.local_node.capability.clone(),
        }
    }

    /// Get mutable reference to local node
    pub fn local_node_mut(&mut self) -> &mut UnifiedNode {
        &mut self.local_node
    }

    /// Get reference to local node
    pub fn local_node(&self) -> &UnifiedNode {
        &self.local_node
    }

    /// Get global job registry
    pub fn global_jobs(&self) -> &HashMap<u64, DistributedJob> {
        &self.global_job_registry
    }

    /// Get mutable reference to global job registry (for testing)
    pub fn global_jobs_mut(&mut self) -> &mut HashMap<u64, DistributedJob> {
        &mut self.global_job_registry
    }
}

/// Network statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub network_block_height: u64,
    pub local_node_stats: crate::node::NodeStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeCapability;

    #[test]
    fn network_coordinator_creation() {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 0,
            reputation: 0,
        };

        let node = UnifiedNode::new("test_node".to_string(), capability, 1000);
        let coordinator = NetworkCoordinator::new(node);

        assert_eq!(coordinator.peer_capabilities.len(), 0);
        assert_eq!(coordinator.global_job_registry.len(), 0);
    }

    #[test]
    fn job_posting_and_volunteering() -> Result<(), crate::node::NodeError> {
        let capability = NodeCapability {
            cpus: 4,
            gpus: 1,
            gpu_memory_gb: 8,
            available_stake: 100,
            reputation: 0,
        };

        // Create job poster
        let poster_node = UnifiedNode::new("poster".to_string(), capability.clone(), 1000);
        let mut poster_coordinator = NetworkCoordinator::new(poster_node);

        // Create worker
        let worker_node = UnifiedNode::new("worker".to_string(), capability.clone(), 1000);
        let mut worker_coordinator = NetworkCoordinator::new(worker_node);
        worker_coordinator.local_node_mut().stake_tokens(150)?;

        // Post job
        let job_message = poster_coordinator.post_job_to_network(
            "Test distributed training".to_string(),
            500,
            capability,
            "data_hash".to_string(),
            "model_spec".to_string(),
            100,
        )?;

        // Worker receives job posting
        let responses = worker_coordinator.handle_message(job_message, "poster")?;
        assert_eq!(responses.len(), 1);

        // Check that worker volunteered
        if let NetworkMessage::JobVolunteer { job_id, node_id, capability: _ } = &responses[0] {
            assert_eq!(*job_id, 1);
            assert_eq!(node_id, "worker");
        } else {
            panic!("Expected JobVolunteer message");
        }

        Ok(())
    }
}
