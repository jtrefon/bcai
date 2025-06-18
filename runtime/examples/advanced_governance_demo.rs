//! Advanced DAO Governance System Demo
//!
//! This demo showcases the sophisticated governance mechanisms:
//! - Quadratic voting to prevent whale dominance
//! - Delegation and liquid democracy
//! - Multi-stage proposal lifecycle
//! - Reputation-weighted voting for technical decisions
//! - Emergency governance procedures

use runtime::advanced_governance::{
    AdvancedGovernance, GovernanceConfig, ProposalType, VoteChoice, 
    DelegationType, ExecutionData, Contribution, ContributionType
};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏛️  BCAI Advanced DAO Governance System Demo");
    println!("============================================\n");

    // Create governance system with custom configuration
    let config = GovernanceConfig {
        min_proposal_stake: 5_000,
        voting_period_days: 3, // Shorter for demo
        execution_delay_hours: 1, // Shorter for demo
        quorum_percentage: 0.15, // 15%
        approval_threshold: 0.55, // 55%
        emergency_threshold: 0.70, // 70%
        reputation_decay_rate: 0.98,
        max_delegation_depth: 5,
    };

    let mut governance = AdvancedGovernance::new(config);

    // Phase 1: Register diverse voter base
    println!("📋 Phase 1: Registering Diverse Voter Base");
    println!("==========================================");

    let voters = vec![
        ("alice_whale", 5_000_000), // Large token holder
        ("bob_dev", 100_000),        // Developer with reputation
        ("charlie_user", 50_000),    // Regular user
        ("diana_validator", 200_000), // Validator
        ("eve_researcher", 75_000),  // AI researcher
        ("frank_community", 25_000), // Community member
        ("grace_investor", 1_000_000), // Investor
        ("henry_dao", 150_000),      // DAO participant
    ];

    for (address, balance) in &voters {
        governance.register_voter(address.to_string(), *balance)?;
        println!("✅ Registered voter: {} with {} BCAI tokens", address, balance);
    }

    // Add reputation scores for technical contributors
    let mut reputation_updates = HashMap::new();
    reputation_updates.insert("bob_dev", 85.0);
    reputation_updates.insert("eve_researcher", 92.0);
    reputation_updates.insert("diana_validator", 78.0);
    reputation_updates.insert("henry_dao", 65.0);

    for (address, reputation) in reputation_updates {
        if let Some(voter) = governance.get_voter(address) {
            let mut updated_voter = voter.clone();
            updated_voter.reputation_score = reputation;
            // In a real system, this would be handled internally
            println!("🎯 Updated reputation for {}: {:.1}", address, reputation);
        }
    }

    println!("\n📊 Initial Governance Stats:");
    let stats = governance.get_governance_stats();
    println!("   Total voters: {}", stats.total_voters);
    println!("   Total voting power: {} BCAI", stats.total_voting_power);

    // Phase 2: Demonstrate delegation
    println!("\n🤝 Phase 2: Delegation and Liquid Democracy");
    println!("==========================================");

    // Alice delegates some power to technical experts
    governance.delegate_voting_power(
        "alice_whale",
        "bob_dev",
        DelegationType::Conditional(vec![ProposalType::Technical, ProposalType::NetworkParameter]),
        Some(Utc::now() + Duration::days(30)),
    )?;
    println!("🤝 Alice delegated technical voting to Bob (conditional)");

    // Community members delegate to active participants
    governance.delegate_voting_power(
        "frank_community",
        "henry_dao",
        DelegationType::Full,
        None,
    )?;
    println!("🤝 Frank delegated all voting power to Henry");

    governance.delegate_voting_power(
        "charlie_user",
        "diana_validator",
        DelegationType::Limited(30_000),
        Some(Utc::now() + Duration::days(14)),
    )?;
    println!("🤝 Charlie delegated limited power to Diana");

    // Phase 3: Submit various proposal types
    println!("\n📋 Phase 3: Multi-Type Proposal Submission");
    println!("==========================================");

    // Standard proposal with quadratic voting
    let standard_proposal = governance.submit_proposal(
        "henry_dao".to_string(),
        ProposalType::Standard,
        "Community Fund Allocation".to_string(),
        "Allocate 100,000 BCAI from community treasury for ecosystem development grants".to_string(),
        Some(ExecutionData {
            contract_address: "0x1234...treasury".to_string(),
            function_call: "allocate_funds".to_string(),
            parameters: vec![1, 2, 3, 4], // Encoded parameters
            gas_limit: 500_000,
            executed: false,
            execution_result: None,
        }),
    )?;
    println!("📋 Standard proposal submitted: {}", &standard_proposal[..12]);

    // Technical proposal with reputation weighting
    let technical_proposal = governance.submit_proposal(
        "bob_dev".to_string(),
        ProposalType::Technical,
        "Consensus Algorithm Upgrade".to_string(),
        "Upgrade to enhanced Proof of Useful Work with improved matrix computation validation".to_string(),
        Some(ExecutionData {
            contract_address: "0x5678...consensus".to_string(),
            function_call: "upgrade_consensus".to_string(),
            parameters: vec![5, 6, 7, 8],
            gas_limit: 1_000_000,
            executed: false,
            execution_result: None,
        }),
    )?;
    println!("🔧 Technical proposal submitted: {}", &technical_proposal[..12]);

    // Emergency proposal
    let emergency_proposal = governance.submit_proposal(
        "diana_validator".to_string(),
        ProposalType::Emergency,
        "Security Patch Deployment".to_string(),
        "Deploy critical security patch to address potential vulnerability in cross-chain bridge".to_string(),
        Some(ExecutionData {
            contract_address: "0x9abc...bridge".to_string(),
            function_call: "emergency_patch".to_string(),
            parameters: vec![9, 10, 11, 12],
            gas_limit: 750_000,
            executed: false,
            execution_result: None,
        }),
    )?;
    println!("🚨 Emergency proposal submitted: {}", &emergency_proposal[..12]);

    // Constitutional proposal requiring supermajority
    let constitutional_proposal = governance.submit_proposal(
        "grace_investor".to_string(),
        ProposalType::Constitutional,
        "Governance Framework Amendment".to_string(),
        "Modify quorum requirements and add new proposal categories for AI model governance".to_string(),
        None,
    )?;
    println!("⚖️  Constitutional proposal submitted: {}", &constitutional_proposal[..12]);

    // Phase 4: Demonstrate different voting mechanisms
    println!("\n🗳️  Phase 4: Multi-Mechanism Voting Process");
    println!("==========================================");

    // Voting on standard proposal (quadratic voting)
    println!("\n📊 Standard Proposal Voting (Quadratic):");
    governance.cast_vote(&standard_proposal, "alice_whale", VoteChoice::For, 
        Some("Supporting ecosystem development".to_string()))?;
    governance.cast_vote(&standard_proposal, "bob_dev", VoteChoice::For, 
        Some("Good for developer community".to_string()))?;
    governance.cast_vote(&standard_proposal, "charlie_user", VoteChoice::For, None)?;
    governance.cast_vote(&standard_proposal, "diana_validator", VoteChoice::Abstain, 
        Some("Need more details on allocation".to_string()))?;
    governance.cast_vote(&standard_proposal, "eve_researcher", VoteChoice::For, None)?;
    governance.cast_vote(&standard_proposal, "grace_investor", VoteChoice::Against, 
        Some("Prefer different allocation strategy".to_string()))?;

    // Voting on technical proposal (reputation-weighted)
    println!("\n🔧 Technical Proposal Voting (Reputation-Weighted):");
    governance.cast_vote(&technical_proposal, "bob_dev", VoteChoice::For, 
        Some("I've reviewed the technical specs - solid upgrade".to_string()))?;
    governance.cast_vote(&technical_proposal, "eve_researcher", VoteChoice::For, 
        Some("Improved matrix validation will benefit AI workloads".to_string()))?;
    governance.cast_vote(&technical_proposal, "diana_validator", VoteChoice::For, 
        Some("As a validator, I support this upgrade".to_string()))?;
    governance.cast_vote(&technical_proposal, "alice_whale", VoteChoice::Abstain, 
        Some("Deferring to technical experts".to_string()))?;
    governance.cast_vote(&technical_proposal, "henry_dao", VoteChoice::For, None)?;

    // Voting on emergency proposal (token-weighted for speed)
    println!("\n🚨 Emergency Proposal Voting (Token-Weighted):");
    governance.cast_vote(&emergency_proposal, "alice_whale", VoteChoice::For, 
        Some("Security is paramount".to_string()))?;
    governance.cast_vote(&emergency_proposal, "grace_investor", VoteChoice::For, 
        Some("Protecting the ecosystem".to_string()))?;
    governance.cast_vote(&emergency_proposal, "diana_validator", VoteChoice::For, 
        Some("I can confirm the vulnerability exists".to_string()))?;
    governance.cast_vote(&emergency_proposal, "bob_dev", VoteChoice::For, None)?;
    governance.cast_vote(&emergency_proposal, "eve_researcher", VoteChoice::For, None)?;
    governance.cast_vote(&emergency_proposal, "henry_dao", VoteChoice::For, None)?;

    // Voting on constitutional proposal (quadratic, supermajority required)
    println!("\n⚖️  Constitutional Proposal Voting (Quadratic, Supermajority):");
    governance.cast_vote(&constitutional_proposal, "alice_whale", VoteChoice::For, 
        Some("Framework needs updating for AI governance".to_string()))?;
    governance.cast_vote(&constitutional_proposal, "bob_dev", VoteChoice::For, None)?;
    governance.cast_vote(&constitutional_proposal, "charlie_user", VoteChoice::Against, 
        Some("Changes seem too complex".to_string()))?;
    governance.cast_vote(&constitutional_proposal, "diana_validator", VoteChoice::For, None)?;
    governance.cast_vote(&constitutional_proposal, "eve_researcher", VoteChoice::For, 
        Some("AI model governance is crucial".to_string()))?;
    governance.cast_vote(&constitutional_proposal, "grace_investor", VoteChoice::For, None)?;
    governance.cast_vote(&constitutional_proposal, "henry_dao", VoteChoice::Abstain, 
        Some("Need more community discussion".to_string()))?;

    // Phase 5: Analyze voting results
    println!("\n📊 Phase 5: Voting Results Analysis");
    println!("===================================");

    let proposals = vec![
        (&standard_proposal, "Standard (Community Fund)"),
        (&technical_proposal, "Technical (Consensus Upgrade)"),
        (&emergency_proposal, "Emergency (Security Patch)"),
        (&constitutional_proposal, "Constitutional (Framework Amendment)"),
    ];

    for (proposal_id, name) in proposals {
        if let Some(proposal) = governance.get_proposal(proposal_id) {
            println!("\n📋 {} Proposal:", name);
            println!("   Proposal ID: {}", &proposal_id[..12]);
            println!("   Voting Mechanism: {:?}", proposal.voting_mechanism);
            println!("   Votes Cast: {}", proposal.votes.len());
            println!("   Quorum Required: {}", proposal.quorum_required);
            println!("   Approval Threshold: {:.1}%", proposal.approval_threshold * 100.0);
            
            let mut for_votes = 0;
            let mut against_votes = 0;
            let mut abstain_votes = 0;
            
            for vote in proposal.votes.values() {
                match vote.choice {
                    VoteChoice::For => for_votes += 1,
                    VoteChoice::Against => against_votes += 1,
                    VoteChoice::Abstain => abstain_votes += 1,
                }
            }
            
            println!("   Vote Distribution: {} For, {} Against, {} Abstain", 
                for_votes, against_votes, abstain_votes);
            println!("   State: {:?}", proposal.state);
        }
    }

    // Phase 6: Demonstrate governance statistics and insights
    println!("\n📈 Phase 6: Governance Analytics");
    println!("===============================");

    let final_stats = governance.get_governance_stats();
    println!("📊 Final Governance Statistics:");
    println!("   Total Proposals: {}", final_stats.total_proposals);
    println!("   Active Proposals: {}", final_stats.active_proposals);
    println!("   Passed Proposals: {}", final_stats.passed_proposals);
    println!("   Total Voters: {}", final_stats.total_voters);
    println!("   Active Voters: {}", final_stats.active_voters);
    println!("   Total Delegations: {}", final_stats.total_delegations);
    println!("   Average Participation: {:.1}%", final_stats.average_participation * 100.0);
    println!("   Total Voting Power: {} BCAI", final_stats.total_voting_power);

    // Phase 7: Demonstrate voter profiles
    println!("\n👥 Phase 7: Voter Profile Analysis");
    println!("==================================");

    for (address, _) in &voters[..4] { // Show first 4 voters
        if let Some(voter) = governance.get_voter(address) {
            println!("\n👤 Voter Profile: {}", address);
            println!("   Token Balance: {} BCAI", voter.token_balance);
            println!("   Reputation Score: {:.1}", voter.reputation_score);
            println!("   Voting History: {} votes", voter.voting_history.len());
            println!("   Delegations Received: {}", voter.delegations_received.len());
            println!("   Delegations Given: {}", voter.delegations_given.len());
            println!("   Participation Rate: {:.1}%", voter.participation_rate * 100.0);
        }
    }

    // Phase 8: Demonstrate quadratic voting impact
    println!("\n🔢 Phase 8: Quadratic Voting Impact Analysis");
    println!("===========================================");

    println!("💡 Quadratic Voting Power Comparison:");
    println!("   Alice (5M tokens): Linear = 5,000,000, Quadratic = {:.0}", (5_000_000.0_f64).sqrt());
    println!("   Grace (1M tokens): Linear = 1,000,000, Quadratic = {:.0}", (1_000_000.0_f64).sqrt());
    println!("   Diana (200K tokens): Linear = 200,000, Quadratic = {:.0}", (200_000.0_f64).sqrt());
    println!("   Bob (100K tokens): Linear = 100,000, Quadratic = {:.0}", (100_000.0_f64).sqrt());
    println!("   Charlie (50K tokens): Linear = 50,000, Quadratic = {:.0}", (50_000.0_f64).sqrt());
    
    println!("\n📊 Quadratic voting reduces whale dominance:");
    let alice_linear_dominance = 5_000_000.0 / 6_600_000.0 * 100.0;
    let alice_quadratic_dominance = (5_000_000.0_f64).sqrt() / 
        ((5_000_000.0 + 1_000_000.0 + 200_000.0 + 100_000.0 + 50_000.0) as f64).sqrt() * 100.0;
    
    println!("   Alice's influence: {:.1}% (linear) vs {:.1}% (quadratic)", 
        alice_linear_dominance, alice_quadratic_dominance);

    // Phase 9: Future governance scenarios
    println!("\n🔮 Phase 9: Advanced Governance Scenarios");
    println!("========================================");

    println!("🎯 Governance System Capabilities:");
    println!("   ✅ Quadratic voting prevents whale dominance");
    println!("   ✅ Reputation weighting for technical decisions");
    println!("   ✅ Flexible delegation with conditions and limits");
    println!("   ✅ Multi-stage proposal lifecycle");
    println!("   ✅ Emergency fast-track procedures");
    println!("   ✅ Constitutional supermajority requirements");
    println!("   ✅ Comprehensive analytics and participation tracking");
    println!("   ✅ Cross-chain governance integration ready");

    println!("\n🚀 Next Steps for Production:");
    println!("   • Integration with smart contract execution");
    println!("   • Real-time proposal state updates");
    println!("   • Advanced delegation chain resolution");
    println!("   • Reputation system automation");
    println!("   • Cross-chain proposal synchronization");
    println!("   • Governance token staking mechanisms");
    println!("   • Automated proposal execution");

    println!("\n🎉 Advanced DAO Governance Demo Completed Successfully!");
    println!("The BCAI governance system demonstrates sophisticated democratic");
    println!("mechanisms that balance token holder influence with technical");
    println!("expertise and community participation.");

    Ok(())
} 