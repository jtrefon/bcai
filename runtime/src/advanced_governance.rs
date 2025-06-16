//! Advanced DAO Governance System for BCAI
//!
//! This module provides sophisticated governance mechanisms:
//! - Quadratic voting to prevent whale dominance
//! - Delegation and liquid democracy
//! - Multi-stage proposal lifecycle
//! - Reputation-weighted voting for technical decisions
//! - Emergency governance procedures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;

// Import blockchain and token systems for integration
use crate::blockchain::{Blockchain, Transaction as BlockchainTransaction};
use crate::token::TokenLedger;
use crate::smart_contracts::SmartContractEngine;

#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("Insufficient voting power: required {required}, available {available}")]
    InsufficientVotingPower { required: u64, available: u64 },
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),
    #[error("Invalid proposal state: {0}")]
    InvalidProposalState(String),
    #[error("Voting period ended")]
    VotingPeriodEnded,
    #[error("Unauthorized action: {0}")]
    Unauthorized(String),
    #[error("Delegation error: {0}")]
    DelegationError(String),
    #[error("Blockchain integration error: {0}")]
    BlockchainError(String),
    #[error("Token ledger error: {0}")]
    TokenError(String),
    #[error("Smart contract error: {0}")]
    ContractError(String),
}

pub type GovernanceResult<T> = Result<T, GovernanceError>;

/// Blockchain integration bridge for governance
#[derive(Debug, Clone)]
pub struct GovernanceBlockchainBridge {
    pub proposal_transactions: HashMap<String, String>, // proposal_id -> tx_hash
    pub vote_transactions: HashMap<String, Vec<String>>, // proposal_id -> vec<vote_tx_hashes>
}

impl GovernanceBlockchainBridge {
    pub fn new() -> Self {
        Self {
            proposal_transactions: HashMap::new(),
            vote_transactions: HashMap::new(),
        }
    }

    /// Record a proposal transaction on blockchain
    pub fn record_proposal_transaction(&mut self, proposal_id: String, tx_hash: String) {
        self.proposal_transactions.insert(proposal_id, tx_hash);
    }

    /// Record a vote transaction on blockchain
    pub fn record_vote_transaction(&mut self, proposal_id: String, tx_hash: String) {
        self.vote_transactions
            .entry(proposal_id)
            .or_insert_with(Vec::new)
            .push(tx_hash);
    }

    /// Get proposal transaction hash
    pub fn get_proposal_transaction(&self, proposal_id: &str) -> Option<&String> {
        self.proposal_transactions.get(proposal_id)
    }

    /// Get vote transaction hashes for a proposal
    pub fn get_vote_transactions(&self, proposal_id: &str) -> Option<&Vec<String>> {
        self.vote_transactions.get(proposal_id)
    }
}

/// Proposal types with different voting mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Standard proposals using quadratic voting
    Standard,
    /// Technical proposals with reputation weighting
    Technical,
    /// Emergency proposals with fast-track voting
    Emergency,
    /// Constitutional changes requiring supermajority
    Constitutional,
    /// Treasury spending proposals
    Treasury,
    /// Network parameter changes
    NetworkParameter,
    /// Cross-chain integration proposals
    CrossChain,
}

/// Proposal lifecycle states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalState {
    Draft,
    Submitted,
    Voting,
    Passed,
    Rejected,
    Executed,
    Cancelled,
    Expired,
}

/// Voting mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingMechanism {
    /// Standard token-weighted voting
    TokenWeighted,
    /// Quadratic voting (sqrt of tokens)
    Quadratic,
    /// Reputation-weighted for technical decisions
    ReputationWeighted,
    /// Hybrid combining multiple mechanisms
    Hybrid { token_weight: f64, reputation_weight: f64 },
}

/// Advanced governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedProposal {
    pub id: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_starts: DateTime<Utc>,
    pub voting_ends: DateTime<Utc>,
    pub execution_delay: u64, // seconds
    pub state: ProposalState,
    pub voting_mechanism: VotingMechanism,
    pub quorum_required: u64,
    pub approval_threshold: f64, // percentage
    pub votes: HashMap<String, Vote>,
    pub delegated_votes: HashMap<String, DelegatedVote>,
    pub execution_data: Option<ExecutionData>,
    pub metadata: HashMap<String, String>,
}

/// Vote with quadratic and delegation support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub choice: VoteChoice,
    pub voting_power: u64,
    pub quadratic_power: f64,
    pub reputation_power: f64,
    pub timestamp: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

/// Delegated voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedVote {
    pub delegator: String,
    pub delegate: String,
    pub voting_power: u64,
    pub delegation_type: DelegationType,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationType {
    /// Delegate all voting power
    Full,
    /// Delegate only specific proposal types
    Conditional(Vec<ProposalType>),
    /// Delegate with maximum power limit
    Limited(u64),
}

/// Execution data for passed proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionData {
    pub contract_address: String,
    pub function_call: String,
    pub parameters: Vec<u8>,
    pub gas_limit: u64,
    pub executed: bool,
    pub execution_result: Option<String>,
}

/// Voter profile with reputation and delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterProfile {
    pub address: String,
    pub token_balance: u64,
    pub reputation_score: f64,
    pub voting_history: Vec<VotingRecord>,
    pub delegations_received: Vec<DelegatedVote>,
    pub delegations_given: Vec<DelegatedVote>,
    pub participation_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingRecord {
    pub proposal_id: String,
    pub vote_choice: VoteChoice,
    pub timestamp: DateTime<Utc>,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub min_proposal_stake: u64,
    pub voting_period_days: u64,
    pub execution_delay_hours: u64,
    pub quorum_percentage: f64,
    pub approval_threshold: f64,
    pub emergency_threshold: f64,
    pub reputation_decay_rate: f64,
    pub max_delegation_depth: u32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            min_proposal_stake: 10_000,
            voting_period_days: 7,
            execution_delay_hours: 24,
            quorum_percentage: 0.1, // 10%
            approval_threshold: 0.5, // 50%
            emergency_threshold: 0.67, // 67%
            reputation_decay_rate: 0.95, // 5% decay per month
            max_delegation_depth: 3,
        }
    }
}

/// Advanced DAO governance system with blockchain integration
pub struct AdvancedGovernance {
    config: GovernanceConfig,
    proposals: HashMap<String, AdvancedProposal>,
    voters: HashMap<String, VoterProfile>,
    delegations: HashMap<String, Vec<DelegatedVote>>,
    reputation_system: ReputationSystem,
    // Blockchain integration components
    blockchain_bridge: GovernanceBlockchainBridge,
    token_ledger: Option<TokenLedger>,
    contract_engine: Option<SmartContractEngine>,
}

impl AdvancedGovernance {
    /// Create new governance system
    pub fn new(config: GovernanceConfig) -> Self {
        Self {
            config,
            proposals: HashMap::new(),
            voters: HashMap::new(),
            delegations: HashMap::new(),
            reputation_system: ReputationSystem::new(),
            blockchain_bridge: GovernanceBlockchainBridge::new(),
            token_ledger: None,
            contract_engine: None,
        }
    }

    /// Create governance system with blockchain integration
    pub fn new_with_blockchain(
        config: GovernanceConfig,
        token_ledger: TokenLedger,
        contract_engine: SmartContractEngine,
    ) -> Self {
        Self {
            config,
            proposals: HashMap::new(),
            voters: HashMap::new(),
            delegations: HashMap::new(),
            reputation_system: ReputationSystem::new(),
            blockchain_bridge: GovernanceBlockchainBridge::new(),
            token_ledger: Some(token_ledger),
            contract_engine: Some(contract_engine),
        }
    }

    /// Verify voting power against token ledger
    pub fn verify_voting_power_from_ledger(&self, voter: &str) -> GovernanceResult<u64> {
        if let Some(ledger) = &self.token_ledger {
            Ok(ledger.balance(voter))
        } else {
            // Fallback to registered balance
            self.voters
                .get(voter)
                .map(|v| v.token_balance)
                .ok_or_else(|| GovernanceError::Unauthorized("Voter not registered".to_string()))
        }
    }

    /// Create blockchain transaction for proposal
    pub fn create_proposal_transaction(&self, proposal: &AdvancedProposal) -> GovernanceResult<BlockchainTransaction> {
        let proposal_data = format!("governance_proposal:{}:{}", proposal.title, proposal.description);
        
        Ok(BlockchainTransaction::JobPosting {
            poster: proposal.proposer.clone(),
            job_spec: proposal_data,
            reward: 0, // Governance proposals don't have rewards
            nonce: 0, // Would be properly calculated in real implementation
        })
    }

    /// Create blockchain transaction for vote
    pub fn create_vote_transaction(&self, proposal_id: &str, vote: &Vote) -> GovernanceResult<BlockchainTransaction> {
        Ok(BlockchainTransaction::ValidationVote {
            validator: vote.voter.clone(),
            job_id: proposal_id.chars().take(8).collect::<String>().parse::<u64>().unwrap_or(0),
            vote: matches!(vote.choice, VoteChoice::For),
            nonce: 0, // Would be properly calculated in real implementation
        })
    }

    /// Register a new voter
    pub fn register_voter(&mut self, address: String, token_balance: u64) -> GovernanceResult<()> {
        let voter = VoterProfile {
            address: address.clone(),
            token_balance,
            reputation_score: 0.0,
            voting_history: Vec::new(),
            delegations_received: Vec::new(),
            delegations_given: Vec::new(),
            participation_rate: 0.0,
        };

        self.voters.insert(address, voter);
        Ok(())
    }

    /// Submit a new proposal with blockchain integration
    pub fn submit_proposal(
        &mut self,
        proposer: String,
        proposal_type: ProposalType,
        title: String,
        description: String,
        execution_data: Option<ExecutionData>,
    ) -> GovernanceResult<String> {
        // Verify proposer voting power from token ledger if available
        let proposer_balance = self.verify_voting_power_from_ledger(&proposer)?;

        if proposer_balance < self.config.min_proposal_stake {
            return Err(GovernanceError::InsufficientVotingPower {
                required: self.config.min_proposal_stake,
                available: proposer_balance,
            });
        }

        // Check proposer registration (for compatibility)
        if !self.voters.contains_key(&proposer) {
            return Err(GovernanceError::Unauthorized("Voter not registered".to_string()));
        }

        // Determine voting mechanism based on proposal type
        let voting_mechanism = match proposal_type {
            ProposalType::Standard => VotingMechanism::Quadratic,
            ProposalType::Technical => VotingMechanism::Hybrid { 
                token_weight: 0.3, 
                reputation_weight: 0.7 
            },
            ProposalType::Emergency => VotingMechanism::TokenWeighted,
            ProposalType::Constitutional => VotingMechanism::Quadratic,
            ProposalType::Treasury => VotingMechanism::Quadratic,
            ProposalType::NetworkParameter => VotingMechanism::Hybrid {
                token_weight: 0.4,
                reputation_weight: 0.6,
            },
            ProposalType::CrossChain => VotingMechanism::Quadratic,
        };

        // Create proposal
        let proposal_id = format!("prop_{}_{}", 
            Utc::now().timestamp_micros(), 
            rand::random::<u32>()
        );

        let now = Utc::now();
        let voting_period = match proposal_type {
            ProposalType::Emergency => chrono::Duration::days(1),
            ProposalType::Constitutional => chrono::Duration::days(14), // Longer for constitutional
            _ => chrono::Duration::days(self.config.voting_period_days as i64),
        };

        let approval_threshold = match proposal_type {
            ProposalType::Constitutional => 0.67, // Supermajority
            ProposalType::Emergency => self.config.emergency_threshold,
            _ => self.config.approval_threshold,
        };

        let proposal = AdvancedProposal {
            id: proposal_id.clone(),
            proposal_type,
            title,
            description,
            proposer,
            created_at: now,
            voting_starts: now, // Start immediately for demo
            voting_ends: now + voting_period,
            execution_delay: self.config.execution_delay_hours * 3600,
            state: ProposalState::Submitted,
            voting_mechanism,
            quorum_required: (self.get_total_voting_power() as f64 * self.config.quorum_percentage) as u64,
            approval_threshold,
            votes: HashMap::new(),
            delegated_votes: HashMap::new(),
            execution_data,
            metadata: HashMap::new(),
        };

        let proposal_title = proposal.title.clone();
        
        // Create blockchain transaction for proposal
        let blockchain_tx = self.create_proposal_transaction(&proposal)?;
        
        // Store the proposal
        self.proposals.insert(proposal_id.clone(), proposal);

        // Record the blockchain transaction (in real implementation, this would submit to blockchain)
        let tx_hash = format!("tx_{}", &proposal_id[..8]);
        self.blockchain_bridge.record_proposal_transaction(proposal_id.clone(), tx_hash.clone());

        println!("📋 Proposal submitted: {} - {}", &proposal_id[..12], proposal_title);
        println!("   Blockchain TX: {}", tx_hash);
        Ok(proposal_id)
    }

    /// Cast vote with blockchain integration
    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        voter: &str,
        choice: VoteChoice,
        reason: Option<String>,
    ) -> GovernanceResult<()> {
        // Check voting period and get voting mechanism first
        let (voting_mechanism, voting_starts, voting_ends) = {
            let proposal = self.proposals.get(proposal_id)
                .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;
            (proposal.voting_mechanism.clone(), proposal.voting_starts, proposal.voting_ends)
        };

        let now = Utc::now();
        if now < voting_starts || now > voting_ends {
            return Err(GovernanceError::VotingPeriodEnded);
        }

        // Verify voter's current token balance from ledger
        let verified_balance = self.verify_voting_power_from_ledger(voter)?;

        // Get voter profile and calculate voting power using verified balance
        let (voting_power, quadratic_power, reputation_power) = {
            let voter_profile = self.voters.get(voter)
                .ok_or_else(|| GovernanceError::Unauthorized("Voter not registered".to_string()))?;
            
            // Use verified balance from token ledger
            let mut temp_profile = voter_profile.clone();
            temp_profile.token_balance = verified_balance;
            
            self.calculate_voting_power(&temp_profile, &voting_mechanism)
        };

        // Create vote (before getting mutable references)
        let vote = Vote {
            voter: voter.to_string(),
            choice: choice.clone(),
            voting_power,
            quadratic_power,
            reputation_power,
            timestamp: now,
            reason,
        };

        // Create blockchain transaction for vote (while we can still immutably borrow self)
        let vote_tx = self.create_vote_transaction(proposal_id, &vote)?;
        let vote_tx_hash = format!("vote_tx_{}_{}_{}", &proposal_id[..8], voter, now.timestamp());

        // Now get mutable reference to proposal
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;

        // Now get mutable reference to voter profile
        let voter_profile = self.voters.get_mut(voter)
            .ok_or_else(|| GovernanceError::Unauthorized("Voter not registered".to_string()))?;
        
        proposal.votes.insert(voter.to_string(), vote);

        // Record vote transaction on blockchain bridge
        self.blockchain_bridge.record_vote_transaction(proposal_id.to_string(), vote_tx_hash.clone());

        // Update voter history
        voter_profile.voting_history.push(VotingRecord {
            proposal_id: proposal_id.to_string(),
            vote_choice: choice,
            timestamp: now,
        });

        // Update participation rate
        self.update_participation_rate(voter);

        // Update proposal state if needed
        self.update_proposal_state(proposal_id)?;

        println!("🗳️  Vote cast: {} on {}", voter, &proposal_id[..12]);
        println!("   Vote TX: {}", vote_tx_hash);
        Ok(())
    }

    /// Delegate voting power
    pub fn delegate_voting_power(
        &mut self,
        delegator: &str,
        delegate: &str,
        delegation_type: DelegationType,
        expires_at: Option<DateTime<Utc>>,
    ) -> GovernanceResult<()> {
        // Prevent self-delegation
        if delegator == delegate {
            return Err(GovernanceError::DelegationError("Cannot delegate to self".to_string()));
        }

        // Check delegation depth
        if self.get_delegation_depth(delegate) >= self.config.max_delegation_depth {
            return Err(GovernanceError::DelegationError("Max delegation depth exceeded".to_string()));
        }

        let delegator_profile = self.voters.get_mut(delegator)
            .ok_or_else(|| GovernanceError::Unauthorized("Delegator not registered".to_string()))?;

        let delegation = DelegatedVote {
            delegator: delegator.to_string(),
            delegate: delegate.to_string(),
            voting_power: delegator_profile.token_balance,
            delegation_type,
            created_at: Utc::now(),
            expires_at,
        };

        // Add to delegator's given delegations
        delegator_profile.delegations_given.push(delegation.clone());

        // Add to delegate's received delegations
        if let Some(delegate_profile) = self.voters.get_mut(delegate) {
            delegate_profile.delegations_received.push(delegation.clone());
        }

        self.delegations.entry(delegate.to_string())
            .or_insert_with(Vec::new)
            .push(delegation);

        println!("🤝 Voting power delegated: {} -> {}", delegator, delegate);
        Ok(())
    }

    /// Execute passed proposal
    pub fn execute_proposal(&mut self, proposal_id: &str) -> GovernanceResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.state != ProposalState::Passed {
            return Err(GovernanceError::InvalidProposalState("Proposal not passed".to_string()));
        }

        // Check execution delay
        let now = Utc::now();
        let execution_time = proposal.voting_ends + chrono::Duration::seconds(proposal.execution_delay as i64);
        if now < execution_time {
            return Err(GovernanceError::InvalidProposalState("Execution delay not met".to_string()));
        }

        // Execute proposal via smart contract engine
        if let Some(execution_data) = &mut proposal.execution_data {
            if let Some(contract_engine) = &self.contract_engine {
                // Create smart contract execution call
                match contract_engine.execute_governance_proposal(proposal_id, execution_data) {
                    Ok(result) => {
                        execution_data.executed = true;
                        execution_data.execution_result = Some(format!("Smart contract executed: {}", result));
                        println!("   ⚡ Smart contract executed successfully");
                    }
                    Err(e) => {
                        execution_data.execution_result = Some(format!("Execution failed: {}", e));
                        return Err(GovernanceError::ContractError(format!("Smart contract execution failed: {}", e)));
                    }
                }
            } else {
                // Fallback for demo mode
                execution_data.executed = true;
                execution_data.execution_result = Some("Executed successfully (demo mode)".to_string());
                println!("   ⚡ Proposal executed in demo mode");
            }
            
            // Update reputation for technical proposals
            if matches!(proposal.proposal_type, ProposalType::Technical) {
                self.reputation_system.add_contribution(Contribution {
                    contributor: proposal.proposer.clone(),
                    contribution_type: ContributionType::GovernanceProposal,
                    impact_score: 10.0,
                    timestamp: now,
                });
            }
        }

        proposal.state = ProposalState::Executed;

        println!("⚡ Proposal executed: {} - {}", &proposal_id[..12], proposal.title);
        Ok(())
    }

    /// Get governance statistics
    pub fn get_governance_stats(&self) -> GovernanceStats {
        let total_proposals = self.proposals.len();
        let active_proposals = self.proposals.values()
            .filter(|p| matches!(p.state, ProposalState::Voting))
            .count();

        let passed_proposals = self.proposals.values()
            .filter(|p| matches!(p.state, ProposalState::Passed | ProposalState::Executed))
            .count();

        let total_voters = self.voters.len();
        let active_voters = self.voters.values()
            .filter(|v| v.participation_rate > 0.1)
            .count();

        let total_delegations = self.delegations.values()
            .map(|d| d.len())
            .sum();

        GovernanceStats {
            total_proposals,
            active_proposals,
            passed_proposals,
            total_voters,
            active_voters,
            total_delegations,
            average_participation: self.calculate_average_participation(),
            total_voting_power: self.get_total_voting_power(),
        }
    }

    /// Get proposal details
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&AdvancedProposal> {
        self.proposals.get(proposal_id)
    }

    /// Get voter profile
    pub fn get_voter(&self, address: &str) -> Option<&VoterProfile> {
        self.voters.get(address)
    }

    /// Update voter token balance
    pub fn update_voter_balance(&mut self, address: &str, new_balance: u64) -> GovernanceResult<()> {
        let voter = self.voters.get_mut(address)
            .ok_or_else(|| GovernanceError::Unauthorized("Voter not registered".to_string()))?;
        
        voter.token_balance = new_balance;
        Ok(())
    }

    // Private helper methods
    fn calculate_voting_power(
        &self,
        voter: &VoterProfile,
        mechanism: &VotingMechanism,
    ) -> (u64, f64, f64) {
        let token_power = voter.token_balance;
        let quadratic_power = (token_power as f64).sqrt();
        let reputation_power = voter.reputation_score;

        match mechanism {
            VotingMechanism::TokenWeighted => (token_power, 0.0, 0.0),
            VotingMechanism::Quadratic => (0, quadratic_power, 0.0),
            VotingMechanism::ReputationWeighted => (0, 0.0, reputation_power),
            VotingMechanism::Hybrid { token_weight, reputation_weight } => {
                let combined = (token_power as f64 * token_weight) + (reputation_power * reputation_weight);
                (0, 0.0, combined)
            }
        }
    }

    fn update_proposal_state(&mut self, proposal_id: &str) -> GovernanceResult<()> {
        // Calculate vote results first
        let (total_votes, approval_votes, quorum_required, approval_threshold, proposal_title) = {
            let proposal = self.proposals.get(proposal_id).unwrap();
            let (total_votes, approval_votes) = self.calculate_vote_results(proposal);
            (total_votes, approval_votes, proposal.quorum_required, proposal.approval_threshold, proposal.title.clone())
        };
        
        let proposal = self.proposals.get_mut(proposal_id).unwrap();
        
        // Check if voting period ended
        if Utc::now() > proposal.voting_ends {
            
            if total_votes >= quorum_required {
                let approval_rate = approval_votes / total_votes as f64;
                if approval_rate >= approval_threshold {
                    proposal.state = ProposalState::Passed;
                    println!("✅ Proposal passed: {} - {}", &proposal_id[..12], proposal_title);
                } else {
                    proposal.state = ProposalState::Rejected;
                    println!("❌ Proposal rejected: {} - {}", &proposal_id[..12], proposal_title);
                }
            } else {
                proposal.state = ProposalState::Rejected; // Failed quorum
                println!("❌ Proposal failed quorum: {} - {}", &proposal_id[..12], proposal_title);
            }
        }

        Ok(())
    }

    fn calculate_vote_results(&self, proposal: &AdvancedProposal) -> (u64, f64) {
        let mut total_votes = 0u64;
        let mut approval_votes = 0.0f64;

        for vote in proposal.votes.values() {
            let vote_weight = match proposal.voting_mechanism {
                VotingMechanism::TokenWeighted => vote.voting_power as f64,
                VotingMechanism::Quadratic => vote.quadratic_power,
                VotingMechanism::ReputationWeighted => vote.reputation_power,
                VotingMechanism::Hybrid { .. } => vote.reputation_power,
            };

            total_votes += vote_weight as u64;
            
            match vote.choice {
                VoteChoice::For => approval_votes += vote_weight,
                VoteChoice::Against => {},
                VoteChoice::Abstain => {},
            }
        }

        (total_votes, approval_votes)
    }

    fn get_delegation_depth(&self, delegate: &str) -> u32 {
        // Simplified delegation depth calculation
        if self.delegations.contains_key(delegate) {
            1 // In reality, would recursively check delegation chains
        } else {
            0
        }
    }

    fn get_total_voting_power(&self) -> u64 {
        self.voters.values().map(|v| v.token_balance).sum()
    }

    fn calculate_average_participation(&self) -> f64 {
        if self.voters.is_empty() {
            return 0.0;
        }
        
        let total_participation: f64 = self.voters.values()
            .map(|v| v.participation_rate)
            .sum();
        
        total_participation / self.voters.len() as f64
    }

    fn update_participation_rate(&mut self, voter_address: &str) {
        if let Some(voter) = self.voters.get_mut(voter_address) {
            let total_proposals = self.proposals.len() as f64;
            let voter_votes = voter.voting_history.len() as f64;
            voter.participation_rate = if total_proposals > 0.0 {
                voter_votes / total_proposals
            } else {
                0.0
            };
        }
    }
}

/// Reputation system for technical governance
pub struct ReputationSystem {
    scores: HashMap<String, f64>,
    contributions: HashMap<String, Vec<Contribution>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub contributor: String,
    pub contribution_type: ContributionType,
    pub impact_score: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContributionType {
    CodeCommit,
    BugReport,
    SecurityAudit,
    Documentation,
    CommunitySupport,
    Research,
    GovernanceProposal,
}

impl ReputationSystem {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            contributions: HashMap::new(),
        }
    }

    pub fn add_contribution(&mut self, contribution: Contribution) {
        let contributor = contribution.contributor.clone();
        
        // Add to contributions
        self.contributions.entry(contributor.clone())
            .or_insert_with(Vec::new)
            .push(contribution.clone());

        // Update reputation score
        let current_score = self.scores.get(&contributor).unwrap_or(&0.0);
        let new_score = current_score + contribution.impact_score;
        self.scores.insert(contributor, new_score);
    }

    pub fn get_reputation(&self, address: &str) -> f64 {
        *self.scores.get(address).unwrap_or(&0.0)
    }

    pub fn decay_reputation(&mut self, decay_rate: f64) {
        for score in self.scores.values_mut() {
            *score *= decay_rate;
        }
    }
}

/// Governance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStats {
    pub total_proposals: usize,
    pub active_proposals: usize,
    pub passed_proposals: usize,
    pub total_voters: usize,
    pub active_voters: usize,
    pub total_delegations: usize,
    pub average_participation: f64,
    pub total_voting_power: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_creation() {
        let config = GovernanceConfig::default();
        let governance = AdvancedGovernance::new(config);
        
        assert_eq!(governance.proposals.len(), 0);
        assert_eq!(governance.voters.len(), 0);
    }

    #[test]
    fn test_voter_registration() {
        let mut governance = AdvancedGovernance::new(GovernanceConfig::default());
        
        let result = governance.register_voter("voter1".to_string(), 10000);
        assert!(result.is_ok());
        assert_eq!(governance.voters.len(), 1);
    }

    #[test]
    fn test_quadratic_voting_power() {
        let governance = AdvancedGovernance::new(GovernanceConfig::default());
        
        let voter = VoterProfile {
            address: "voter1".to_string(),
            token_balance: 10000,
            reputation_score: 0.8,
            voting_history: Vec::new(),
            delegations_received: Vec::new(),
            delegations_given: Vec::new(),
            participation_rate: 0.5,
        };

        let (_, quadratic_power, _) = governance.calculate_voting_power(
            &voter, 
            &VotingMechanism::Quadratic
        );

        assert_eq!(quadratic_power, 100.0); // sqrt(10000) = 100
    }

    #[test]
    fn test_proposal_submission() {
        let mut governance = AdvancedGovernance::new(GovernanceConfig::default());
        
        // Register voter with sufficient stake
        governance.register_voter("proposer".to_string(), 50000).unwrap();
        
        let result = governance.submit_proposal(
            "proposer".to_string(),
            ProposalType::Standard,
            "Test Proposal".to_string(),
            "A test proposal for the governance system".to_string(),
            None,
        );
        
        assert!(result.is_ok());
        assert_eq!(governance.proposals.len(), 1);
    }
} 