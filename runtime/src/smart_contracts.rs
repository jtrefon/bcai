use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Contract not found: {0}")]
    ContractNotFound(String),
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    #[error("Unauthorized access for address: {0}")]
    UnauthorizedAccess(String),
    #[error("Invalid contract state: {0}")]
    InvalidState(String),
    #[error("Contract execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Job requirements not met: {0}")]
    JobRequirementsNotMet(String),
    #[error("Staking error: {0}")]
    StakingError(String),
}

pub type ContractResult<T> = Result<T, ContractError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractType {
    AIJob,
    Staking,
    Governance,
    CrossChain,
    DataMarket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractStatus {
    Created,
    Active,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddress(pub String);

impl ContractAddress {
    pub fn new(prefix: &str) -> Self {
        let timestamp = Utc::now().timestamp_micros();
        let random = rand::random::<u32>();
        Self(format!("{}_{}_{}", prefix, timestamp, random))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIJobRequirements {
    pub min_accuracy: f64,
    pub max_training_time: u64, // seconds
    pub required_data_size: u64,
    pub model_type: String,
    pub privacy_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIJobResult {
    pub accuracy: f64,
    pub training_time: u64,
    pub model_hash: String,
    pub validation_score: f64,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIJobContract {
    pub address: ContractAddress,
    pub client: String,
    pub reward_pool: u64,
    pub requirements: AIJobRequirements,
    pub assigned_nodes: Vec<String>,
    pub status: ContractStatus,
    pub created_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub result: Option<AIJobResult>,
    pub escrow_released: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingContract {
    pub address: ContractAddress,
    pub staker: String,
    pub amount: u64,
    pub lock_period: u64, // seconds
    pub reward_rate: f64, // annual percentage
    pub status: ContractStatus,
    pub created_at: DateTime<Utc>,
    pub unlock_at: DateTime<Utc>,
    pub accumulated_rewards: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub voting_power_required: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub created_at: DateTime<Utc>,
    pub voting_deadline: DateTime<Utc>,
    pub executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContract {
    pub address: ContractAddress,
    pub proposals: HashMap<String, GovernanceProposal>,
    pub total_voting_power: u64,
    pub min_proposal_stake: u64,
    pub voting_period_days: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub address: ContractAddress,
    pub contract_type: ContractType,
    pub owner: String,
    pub status: ContractStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ai_job: Option<AIJobContract>,
    pub staking: Option<StakingContract>,
    pub governance: Option<GovernanceContract>,
}

pub struct SmartContractEngine {
    contracts: HashMap<String, SmartContract>,
    balances: HashMap<String, u64>,
    staking_pools: HashMap<String, u64>,
    total_staked: u64,
    governance_threshold: u64,
}

impl SmartContractEngine {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            balances: HashMap::new(),
            staking_pools: HashMap::new(),
            total_staked: 0,
            governance_threshold: 100000, // Minimum tokens for governance participation
        }
    }

    // ============================================================================
    // AI Job Contract Functions
    // ============================================================================

    pub fn create_ai_job_contract(
        &mut self,
        client: String,
        reward_amount: u64,
        requirements: AIJobRequirements,
        deadline_hours: u64,
    ) -> ContractResult<ContractAddress> {
        let client_balance = self.balances.get(&client).unwrap_or(&0);
        if *client_balance < reward_amount {
            return Err(ContractError::InsufficientBalance {
                required: reward_amount,
                available: *client_balance,
            });
        }

        let address = ContractAddress::new("aijob");
        let now = Utc::now();
        let deadline = now + chrono::Duration::hours(deadline_hours as i64);

        let ai_job = AIJobContract {
            address: address.clone(),
            client: client.clone(),
            reward_pool: reward_amount,
            requirements,
            assigned_nodes: Vec::new(),
            status: ContractStatus::Created,
            created_at: now,
            deadline,
            result: None,
            escrow_released: false,
        };

        let contract = SmartContract {
            address: address.clone(),
            contract_type: ContractType::AIJob,
            owner: client.clone(),
            status: ContractStatus::Created,
            created_at: now,
            updated_at: now,
            ai_job: Some(ai_job),
            staking: None,
            governance: None,
        };

        // Lock funds in escrow
        self.balances.insert(client.clone(), client_balance - reward_amount);
        self.contracts.insert(address.0.clone(), contract);

        Ok(address)
    }

    pub fn assign_job_to_nodes(
        &mut self,
        contract_address: &str,
        node_ids: Vec<String>,
    ) -> ContractResult<()> {
        let contract = self
            .contracts
            .get_mut(contract_address)
            .ok_or_else(|| ContractError::ContractNotFound(contract_address.to_string()))?;

        if let Some(ai_job) = &mut contract.ai_job {
            if matches!(ai_job.status, ContractStatus::Created) {
                ai_job.assigned_nodes = node_ids;
                ai_job.status = ContractStatus::Active;
                contract.status = ContractStatus::Active;
                contract.updated_at = Utc::now();
                Ok(())
            } else {
                Err(ContractError::InvalidState("Job is not in Created state".to_string()))
            }
        } else {
            Err(ContractError::InvalidState("Not an AI job contract".to_string()))
        }
    }

    pub fn submit_job_result(
        &mut self,
        contract_address: &str,
        node_id: &str,
        result: AIJobResult,
    ) -> ContractResult<bool> {
        let contract = self
            .contracts
            .get_mut(contract_address)
            .ok_or_else(|| ContractError::ContractNotFound(contract_address.to_string()))?;

        if let Some(ai_job) = &mut contract.ai_job {
            if !ai_job.assigned_nodes.contains(&node_id.to_string()) {
                return Err(ContractError::UnauthorizedAccess(node_id.to_string()));
            }

            if matches!(ai_job.status, ContractStatus::Active) {
                // Validate results against requirements
                if result.accuracy < ai_job.requirements.min_accuracy {
                    return Err(ContractError::JobRequirementsNotMet(format!(
                        "Accuracy {} below minimum {}",
                        result.accuracy, ai_job.requirements.min_accuracy
                    )));
                }

                if result.training_time > ai_job.requirements.max_training_time {
                    return Err(ContractError::JobRequirementsNotMet(format!(
                        "Training time {} exceeds maximum {}",
                        result.training_time, ai_job.requirements.max_training_time
                    )));
                }

                ai_job.result = Some(result);
                ai_job.status = ContractStatus::Completed;
                contract.status = ContractStatus::Completed;
                contract.updated_at = Utc::now();

                // Release escrow and distribute rewards
                let reward_per_node = ai_job.reward_pool / ai_job.assigned_nodes.len() as u64;
                let assigned_nodes = ai_job.assigned_nodes.clone();

                // Mark escrow as released
                ai_job.escrow_released = true;

                // Distribute rewards after releasing the mutable borrow
                let _ = contract;
                for node_id in &assigned_nodes {
                    let current_balance = self.balances.get(node_id).unwrap_or(&0);
                    self.balances.insert(node_id.clone(), current_balance + reward_per_node);
                }

                Ok(true)
            } else {
                Err(ContractError::InvalidState("Job is not active".to_string()))
            }
        } else {
            Err(ContractError::InvalidState("Not an AI job contract".to_string()))
        }
    }

    // ============================================================================
    // Staking Contract Functions
    // ============================================================================

    pub fn create_staking_contract(
        &mut self,
        staker: String,
        amount: u64,
        lock_period_days: u64,
        reward_rate: f64,
    ) -> ContractResult<ContractAddress> {
        let staker_balance = self.balances.get(&staker).unwrap_or(&0);
        if *staker_balance < amount {
            return Err(ContractError::InsufficientBalance {
                required: amount,
                available: *staker_balance,
            });
        }

        let address = ContractAddress::new("stake");
        let now = Utc::now();
        let unlock_at = now + chrono::Duration::days(lock_period_days as i64);

        let staking = StakingContract {
            address: address.clone(),
            staker: staker.clone(),
            amount,
            lock_period: lock_period_days * 24 * 3600,
            reward_rate,
            status: ContractStatus::Active,
            created_at: now,
            unlock_at,
            accumulated_rewards: 0,
        };

        let contract = SmartContract {
            address: address.clone(),
            contract_type: ContractType::Staking,
            owner: staker.clone(),
            status: ContractStatus::Active,
            created_at: now,
            updated_at: now,
            ai_job: None,
            staking: Some(staking),
            governance: None,
        };

        // Lock staking amount
        self.balances.insert(staker.clone(), staker_balance - amount);
        self.total_staked += amount;
        let pool_balance = self.staking_pools.get(&staker).unwrap_or(&0);
        self.staking_pools.insert(staker, pool_balance + amount);

        self.contracts.insert(address.0.clone(), contract);

        Ok(address)
    }

    pub fn calculate_staking_rewards(&self, contract_address: &str) -> ContractResult<u64> {
        let contract = self
            .contracts
            .get(contract_address)
            .ok_or_else(|| ContractError::ContractNotFound(contract_address.to_string()))?;

        if let Some(staking) = &contract.staking {
            let now = Utc::now();
            let time_staked = (now - staking.created_at).num_seconds() as u64;
            let annual_seconds = 365 * 24 * 3600;

            let rewards = (staking.amount as f64 * staking.reward_rate * time_staked as f64)
                / annual_seconds as f64;

            Ok(rewards as u64)
        } else {
            Err(ContractError::InvalidState("Not a staking contract".to_string()))
        }
    }

    pub fn unstake(&mut self, contract_address: &str) -> ContractResult<u64> {
        let contract = self
            .contracts
            .get_mut(contract_address)
            .ok_or_else(|| ContractError::ContractNotFound(contract_address.to_string()))?;

        if let Some(staking) = &mut contract.staking {
            let now = Utc::now();
            if now < staking.unlock_at {
                return Err(ContractError::StakingError(format!(
                    "Staking period not completed. Unlock at: {}",
                    staking.unlock_at
                )));
            }

            // Calculate rewards before modifying contract
            let now = Utc::now();
            let time_staked = (now - staking.created_at).num_seconds() as u64;
            let annual_seconds = 365 * 24 * 3600;
            let rewards = (staking.amount as f64 * staking.reward_rate * time_staked as f64)
                / annual_seconds as f64;
            let rewards = rewards as u64;

            let total_return = staking.amount + rewards;
            let staker_address = staking.staker.clone();
            let staking_amount = staking.amount;

            // Update contract status
            staking.status = ContractStatus::Completed;
            staking.accumulated_rewards = rewards;
            contract.status = ContractStatus::Completed;
            contract.updated_at = now;

            // Release the mutable borrow before updating balances
            let _ = contract;

            // Return staked amount plus rewards
            let current_balance = self.balances.get(&staker_address).unwrap_or(&0);
            self.balances.insert(staker_address.clone(), current_balance + total_return);

            // Update staking pools
            self.total_staked -= staking_amount;
            let pool_balance = self.staking_pools.get(&staker_address).unwrap_or(&0);
            if *pool_balance >= staking_amount {
                self.staking_pools.insert(staker_address, pool_balance - staking_amount);
            }

            Ok(total_return)
        } else {
            Err(ContractError::InvalidState("Not a staking contract".to_string()))
        }
    }

    // ============================================================================
    // Governance Contract Functions
    // ============================================================================

    pub fn create_governance_contract(
        &mut self,
        creator: String,
    ) -> ContractResult<ContractAddress> {
        let creator_balance = self.balances.get(&creator).unwrap_or(&0);
        if *creator_balance < self.governance_threshold {
            return Err(ContractError::InsufficientBalance {
                required: self.governance_threshold,
                available: *creator_balance,
            });
        }

        let address = ContractAddress::new("gov");
        let now = Utc::now();

        let governance = GovernanceContract {
            address: address.clone(),
            proposals: HashMap::new(),
            total_voting_power: self.total_staked,
            min_proposal_stake: 10000,
            voting_period_days: 7,
        };

        let contract = SmartContract {
            address: address.clone(),
            contract_type: ContractType::Governance,
            owner: creator,
            status: ContractStatus::Active,
            created_at: now,
            updated_at: now,
            ai_job: None,
            staking: None,
            governance: Some(governance),
        };

        self.contracts.insert(address.0.clone(), contract);
        Ok(address)
    }

    pub fn create_proposal(
        &mut self,
        contract_address: &str,
        proposer: String,
        title: String,
        description: String,
    ) -> ContractResult<String> {
        let proposer_stake = self.staking_pools.get(&proposer).unwrap_or(&0);
        if *proposer_stake < 10000 {
            return Err(ContractError::InsufficientBalance {
                required: 10000,
                available: *proposer_stake,
            });
        }

        let contract = self
            .contracts
            .get_mut(contract_address)
            .ok_or_else(|| ContractError::ContractNotFound(contract_address.to_string()))?;

        if let Some(governance) = &mut contract.governance {
            let proposal_id = format!("prop_{}_{}", Utc::now().timestamp(), rand::random::<u32>());
            let now = Utc::now();
            let deadline = now + chrono::Duration::days(governance.voting_period_days as i64);

            let proposal = GovernanceProposal {
                id: proposal_id.clone(),
                title,
                description,
                proposer,
                voting_power_required: governance.total_voting_power / 2, // 50% threshold
                votes_for: 0,
                votes_against: 0,
                created_at: now,
                voting_deadline: deadline,
                executed: false,
            };

            governance.proposals.insert(proposal_id.clone(), proposal);
            contract.updated_at = now;

            Ok(proposal_id)
        } else {
            Err(ContractError::InvalidState("Not a governance contract".to_string()))
        }
    }

    pub fn vote_on_proposal(
        &mut self,
        contract_address: &str,
        proposal_id: &str,
        voter: &str,
        vote_for: bool,
    ) -> ContractResult<()> {
        let voter_stake = self.staking_pools.get(voter).unwrap_or(&0);
        if *voter_stake == 0 {
            return Err(ContractError::UnauthorizedAccess("No staking power".to_string()));
        }

        let contract = self
            .contracts
            .get_mut(contract_address)
            .ok_or_else(|| ContractError::ContractNotFound(contract_address.to_string()))?;

        if let Some(governance) = &mut contract.governance {
            let proposal = governance.proposals.get_mut(proposal_id).ok_or_else(|| {
                ContractError::ContractNotFound(format!("Proposal {}", proposal_id))
            })?;

            if Utc::now() > proposal.voting_deadline {
                return Err(ContractError::InvalidState("Voting period ended".to_string()));
            }

            if vote_for {
                proposal.votes_for += voter_stake;
            } else {
                proposal.votes_against += voter_stake;
            }

            contract.updated_at = Utc::now();
            Ok(())
        } else {
            Err(ContractError::InvalidState("Not a governance contract".to_string()))
        }
    }

    // ============================================================================
    // Utility Functions
    // ============================================================================

    pub fn get_contract(&self, address: &str) -> Option<&SmartContract> {
        self.contracts.get(address)
    }

    pub fn get_balance(&self, address: &str) -> u64 {
        *self.balances.get(address).unwrap_or(&0)
    }

    pub fn set_balance(&mut self, address: String, amount: u64) {
        self.balances.insert(address, amount);
    }

    pub fn get_total_staked(&self) -> u64 {
        self.total_staked
    }

    pub fn get_active_contracts(&self) -> Vec<&SmartContract> {
        self.contracts.values().filter(|c| matches!(c.status, ContractStatus::Active)).collect()
    }

    pub fn get_contracts_by_type(&self, contract_type: ContractType) -> Vec<&SmartContract> {
        self.contracts.values().filter(|c| c.contract_type == contract_type).collect()
    }

    // ============================================================================
    // Governance Integration Functions
    // ============================================================================

    /// Execute a governance proposal through smart contract system
    pub fn execute_governance_proposal(
        &self,
        proposal_id: &str,
        execution_data: &crate::advanced_governance::ExecutionData,
    ) -> ContractResult<String> {
        match execution_data.contract_address.as_str() {
            "0x1234...treasury" => {
                // Treasury allocation
                self.execute_treasury_allocation(proposal_id, execution_data)
            }
            "0x5678...consensus" => {
                // Consensus upgrade
                self.execute_consensus_upgrade(proposal_id, execution_data)
            }
            "0x9abc...bridge" => {
                // Cross-chain bridge operations
                self.execute_bridge_operation(proposal_id, execution_data)
            }
            _ => {
                // Generic smart contract execution
                self.execute_generic_contract(proposal_id, execution_data)
            }
        }
    }

    fn execute_treasury_allocation(
        &self,
        _proposal_id: &str,
        _execution_data: &crate::advanced_governance::ExecutionData,
    ) -> ContractResult<String> {
        // Simulate treasury fund allocation
        let allocation_amount = 100_000; // Would be parsed from execution_data.parameters
        
        // Simulate treasury fund allocation (in real system, this would transfer tokens)
        println!("   💰 Allocating {} tokens from treasury to ecosystem development", allocation_amount);
        println!("   💰 Treasury allocation executed successfully");

        Ok(format!("Treasury allocation executed: {} tokens allocated to ecosystem development", allocation_amount))
    }

    fn execute_consensus_upgrade(
        &self,
        _proposal_id: &str,
        _execution_data: &crate::advanced_governance::ExecutionData,
    ) -> ContractResult<String> {
        // Simulate consensus algorithm upgrade
        println!("   🔧 Upgrading consensus algorithm...");
        println!("   🔧 Implementing enhanced Proof of Useful Work");
        println!("   🔧 Upgrading matrix computation validation");

        // In real implementation, this would trigger network-wide upgrade
        Ok("Consensus algorithm upgraded successfully".to_string())
    }

    fn execute_bridge_operation(
        &self,
        _proposal_id: &str,
        _execution_data: &crate::advanced_governance::ExecutionData,
    ) -> ContractResult<String> {
        // Simulate emergency bridge patch
        println!("   🚨 Applying emergency security patch to cross-chain bridge");
        println!("   🚨 Updating bridge validation logic");
        println!("   🚨 Strengthening cross-chain transaction verification");

        // In real implementation, this would patch bridge contracts
        Ok("Emergency bridge patch applied successfully".to_string())
    }

    fn execute_generic_contract(
        &self,
        _proposal_id: &str,
        execution_data: &crate::advanced_governance::ExecutionData,
    ) -> ContractResult<String> {
        // Generic contract execution
        let function_call = &execution_data.function_call;
        let gas_limit = execution_data.gas_limit;

        // Simulate contract execution
        println!("   ⚡ Executing contract function: {}", function_call);
        println!("   ⚡ Gas limit: {}", gas_limit);

        // In real implementation, this would execute arbitrary smart contract functions
        Ok(format!("Contract function '{}' executed successfully", function_call))
    }
}

impl Default for SmartContractEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ai_job_contract() {
        let mut engine = SmartContractEngine::new();
        engine.set_balance("client1".to_string(), 100000);

        let requirements = AIJobRequirements {
            min_accuracy: 0.85,
            max_training_time: 3600,
            required_data_size: 1000,
            model_type: "neural_network".to_string(),
            privacy_level: "high".to_string(),
        };

        let result = engine.create_ai_job_contract("client1".to_string(), 50000, requirements, 24);

        assert!(result.is_ok());
        assert_eq!(engine.get_balance("client1"), 50000); // Escrow locked
    }

    #[test]
    fn test_staking_contract() {
        let mut engine = SmartContractEngine::new();
        engine.set_balance("staker1".to_string(), 100000);

        let result = engine.create_staking_contract(
            "staker1".to_string(),
            75000,
            30,   // 30 days
            0.15, // 15% APR
        );

        assert!(result.is_ok());
        assert_eq!(engine.get_balance("staker1"), 25000);
        assert_eq!(engine.get_total_staked(), 75000);
    }

    #[test]
    fn test_governance_proposal() {
        let mut engine = SmartContractEngine::new();
        engine.set_balance("creator".to_string(), 300000); // Increased balance

        // Create staking to have voting power
        let _staking = engine
            .create_staking_contract(
                "creator".to_string(),
                100000, // Reduced stake amount
                90,
                0.12,
            )
            .unwrap();

        let gov_result = engine.create_governance_contract("creator".to_string());
        assert!(gov_result.is_ok());

        let gov_address = gov_result.unwrap();
        let proposal_result = engine.create_proposal(
            &gov_address.0,
            "creator".to_string(),
            "Increase reward rate".to_string(),
            "Proposal to increase staking rewards by 2%".to_string(),
        );

        assert!(proposal_result.is_ok());
    }

    #[test]
    fn test_insufficient_balance() {
        let mut engine = SmartContractEngine::new();
        engine.set_balance("poor_client".to_string(), 1000);

        let requirements = AIJobRequirements {
            min_accuracy: 0.8,
            max_training_time: 3600,
            required_data_size: 500,
            model_type: "linear".to_string(),
            privacy_level: "medium".to_string(),
        };

        let result = engine.create_ai_job_contract(
            "poor_client".to_string(),
            50000, // More than balance
            requirements,
            24,
        );

        assert!(matches!(result, Err(ContractError::InsufficientBalance { .. })));
    }
}
