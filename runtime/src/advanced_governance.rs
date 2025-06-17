//! Advanced DAO Governance System for BCAI
//!
//! This module provides sophisticated governance mechanisms:
//! - Quadratic voting to prevent whale dominance
//! - Delegation and liquid democracy
//! - Multi-stage proposal lifecycle
//! - Reputation-weighted voting for technical decisions
//! - Emergency governance procedures

use std::collections::{HashMap, BTreeSet};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::time::Duration;

// --- Core Data Structures for Advanced Governance ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub proposer: String, // Public key of the proposer
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub voting_start_time: chrono::DateTime<chrono::Utc>,
    pub voting_end_time: chrono::DateTime<chrono::Utc>,
    pub votes: Votes,
    pub execution_details: Option<ExecutionDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Active,  // Currently in voting period
    Succeeded,
    Defeated,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    ParameterChange {
        parameter: String,
        new_value: serde_json::Value,
    },
    CodeUpgrade {
        git_commit_hash: String,
        upgrade_height: u64, // Block height to apply the upgrade
    },
    Grant {
        recipient: String,
        amount: u64,
        currency: String,
    },
    CommunityPoolSpend {
        recipient: String,
        amount: u64,
    },
    PlainText, // For general signaling proposals
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Votes {
    pub yes: u64,
    pub no: u64,
    pub abstain: u64,
    pub no_with_veto: u64,
    pub voters: HashMap<String, VoteOption>, // voter_pubkey -> vote
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDetails {
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub transaction_hash: String,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceParameters {
    pub min_deposit: u64,
    pub max_deposit_period: Duration,
    pub voting_period: Duration,
    pub quorum: f64, // The minimum percentage of total voting power that must vote
    pub threshold: f64, // The minimum percentage of 'Yes' votes for a proposal to pass
    pub veto_threshold: f64, // The minimum percentage of 'NoWithVeto' votes to veto a proposal
}

// NOTE: Removed placeholder implementation structs:
// - GovernanceSystem
// - VotingModule
// - TreasuryModule
// This file now only defines the data models for advanced governance.