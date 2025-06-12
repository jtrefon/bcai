Project Plan and Roadmap
Below is the project plan and development roadmap, outlining milestones from the initial design to mainnet launch. The roadmap is presented in phases with key goals, deliverables, and tasks in each. We use a checklist/todo format for clarity, which can be tracked and updated as work progresses.
Milestone 1: Research & Whitepaper (Q4 2025)
 Literature Review: Gather and study research on PoUW, decentralized ML (Completed initial review of PoUW paper
arxiv.org
, Bittensor
arxiv.org
, etc.).
 Concept Validation: Discuss feasibility of proof-of-training consensus with experts (Done via preliminary consultations).
 Whitepaper Draft: Write and publish the formal whitepaper covering vision, architecture, and economics (This document serves that role – Completed).
 Community Feedback: Solicit feedback on whitepaper and incorporate revisions (In progress via RFC-001 and outreach).
Milestone 2: Prototype Development (Q1 2026)
 Choose Base Platform: Decide on building atop Substrate vs. forking an EVM chain. (Tentative: Substrate chosen for flexibility.)
 Devnet Setup: Initialize a development blockchain network with basic token and simple PoS consensus to act as a playground.
 Basic Job Contract: Implement a rudimentary JobManager contract/pallet where tasks can be posted and claimed (without full verification logic yet).
 GPU Integration Spike: Prototype executing a dummy ML task as part of block production in a dev environment (e.g., integrate a GPU computation in the mining loop in a controlled setting). This will help identify API needs for GPU calls.
 CLI and SDK: Start developing a client SDK for job submission (e.g., a Python library that researchers can use to submit training jobs easily).
 Documentation: Set up initial docs for developers (how to run a node, how to submit a job).
(Goal: By end of Q1 2026, have a minimal “hello world” version of the network where a simple ML task – like training a small model on dummy data – can be executed and verified manually.)
Milestone 3: Testnet Alpha – “Proof of Concept” (Q2 2026)
 Consensus Algorithm (Alpha): Implement the first version of Proof-of-Useful-Work consensus. Likely a simplified version: e.g., block proposers must include a hash of some computation – initially this could be something like a matrix multiplication as a placeholder for training. Over time this will be replaced with real training logic.
 Node Software v0.1: Develop the node program with roles:
 Trainer: able to load a predefined model and dataset and perform computation.
 Evaluator: able to verify results (for now, perhaps just recompute a known result or check a hash).
 Basic P2P networking for those roles.
 Smart Contracts (Alpha): Deploy contracts for JobManager, WorkerRegistry, etc., on the testnet with limited functionality (maybe skipping slashing in alpha).
 Frontend/Explorer: Set up a block explorer and a dashboard to monitor training jobs on the testnet (so we can visualize tasks, participants, results).
 Internal Testing: Run a closed testnet with a few nodes on our team’s machines. Post test jobs (e.g., train a small logistic regression) and ensure the flow from posting to result works, even if some steps are manual.
 Bug Fixes & Optimization: Collect logs, identify bottlenecks (network lag, memory issues with model handling, etc.), and improve the code.
(Goal: By end of Q2 2026, have an internal alpha testnet where basic useful work is part of block production, even if simplified. We should be able to demonstrate end-to-end a task getting done on the network.)
Milestone 4: Testnet Beta – Public Launch (Q3 2026)
 Security Enhancements: Add slashing conditions and reputation tracking to penalize misbehavior on testnet. This includes implementing the Evaluator logic fully: e.g., using a test dataset to validate models.
 Real Training Integration: Swap out placeholder work with actual training for a known model (perhaps something manageable like training an MNIST classifier). This will involve integrating a machine learning library (maybe PyTorch with CUDA) into the node software. Milestone: a block is mined because a model training reached a certain accuracy, not just a dummy calculation.
 Open Testnet Release: Open up the testnet for outside nodes. Invite community members to run nodes (with incentivization via testnet tokens or rewards).
 Cross-Chain Bridge Demo: If possible, deploy a simple bridge contract on Ethereum Ropsten (or successor) so that we can lock some ETH and mint test TRAIN on our network. This is a stretch goal for testnet but good to test early.
 Documentation & Tutorials: By now, update the documentation for public users – how to join the testnet, how to submit an AI job (with examples), how to write a custom job (if applicable). Provide sample code for a client that posts a training job.
 Community Building: Run an incentivized testnet challenge – e.g., “train XYZ model on our network and win mainnet tokens in the future”. This will attract more testers and also stress-test the system with real use cases.
(Goal: During Q3 2026, a stable public testnet is live, with community participation, running real AI training tasks on-chain. Aim for at least e.g. 100 distinct nodes participating and several example models successfully trained.)
Milestone 5: Audit, Optimization, and Hardening (Q4 2026)
 External Security Audit: Engage professionals to audit the smart contracts and the node software (consensus logic especially) for vulnerabilities. Address all critical issues found.
 Performance Tuning: Profile the network under load. Optimize critical paths (for example, ensure that the overhead of our consensus doesn’t slow block times too much, tweak block size or training iteration per block to find a sweet spot). Possibly implement parallelism or better networking (like switching to UDP for gradient exchange if needed).
 Scalability Testing: Simulate large jobs or multiple concurrent jobs. See how the network handles it. This might involve spinning up dozens of nodes in cloud to simulate a heavy scenario. Use results to inform whether sharding or splitting tasks is needed sooner.
 Economic Parameter Lockdown: Using testnet data, finalize parameters for mainnet: initial token supply or distribution plan, block reward schedule, staking requirements, etc. Possibly run a testnet economic simulation to ensure no unforeseen issues (like inflation too high/low, or weird incentive failures).
 Governance Bootstrapping: By this time, set up the DAO governance contracts and have a plan to seed initial governance (maybe a council for earliest weeks, then hand over to token holder voting as tokens distribute). Do a trial governance vote on testnet to ensure process works.
(Goal: Ensure that by end of 2026, the network is secure, robust, and performing well in a variety of conditions – and that we are confident to launch mainnet.)
Milestone 6: Mainnet Launch (Q1 2027)
 Genesis Block Preparation: Configure the genesis block for mainnet. This includes allocation of any initial tokens (for project treasury, early backers, airdrops to testnet participants, etc.), and setting initial network parameters.
 Mainnet Deployment: Launch the mainnet nodes and create the genesis block. Early validators/miners (likely including the core team and selected community members) will start producing blocks and stabilize the chain.
 Public Announcement & Listing: Announce the mainnet launch publicly. Work on getting the token listed on exchanges (if not already) for broader distribution, so users can acquire tokens to use the network.
 Developer Outreach: Mainnet is live – begin outreach to AI researchers, startups, and open-source AI projects to use the network. Provide support for first real use cases migrating to mainnet. Possibly host a hackathon for building apps/services on top of the network (e.g., an AI marketplace or a decentralized Kaggle concept).
 Monitor and Maintain: Closely monitor network health in initial weeks. Be ready to issue patches for any minor bugs. Mainnet should have an upgrade path (via governance or emergency security council) in case hotfixes are needed.
(Goal: A functioning Mainnet v1.0 running the decentralized AI training protocol, with an initial set of validators and users performing real training tasks in production.)
Milestone 7: Post-Launch Improvements (Q2 2027 and beyond)
 Decentralization Increase: Gradually add more validators, encourage more independent node operators to join. Possibly reach out to miners from other PoW coins (GPUs) to repurpose to our network.
 Feature Upgrades: Through governance, propose and implement upgrades such as improved PoUW (maybe integrating new research), support for more ML frameworks (e.g., TensorFlow in addition to PyTorch), adding a Python smart contract execution module for even smoother user experience.
 Ecosystem Growth: Promote development of tooling: wallets supporting our chain, AI model hubs linking to our results, integration with AI workflows (plugins for TensorBoard or PyTorch Lightning, for instance, to offload training to our network).
 Bridge Expansion: Deploy robust bridges to major chains so assets and data can flow. Possibly explore connecting to Polkadot/Kusama if our chain could be a parachain for shared security.
 Research Collaborations: Partner with academic or corporate research labs to run experiments on the network. Their feedback can guide next steps (like what consensus tweaks would better fit large model training, etc.).
 Community Governance Handover: If not fully decentralized yet, aim to hand off any remaining control (like upgrade keys) to the DAO or multi-sig with community representation. The project should transition to community-driven mode, with the founding team taking a step back into contributors among many.
This roadmap is ambitious and subject to change as we encounter new challenges or discoveries. Each milestone will be refined and detailed further as we approach it. We will keep this roadmap updated (likely in our project README or wiki) with progress and adjust timelines if necessary. The development will be done in the open, with regular updates posted for the community (e.g., monthly progress reports, testnet stats, etc.). By following this roadmap, we plan to move methodically from concept to reality, ensuring at each step that we incorporate feedback and maintain the core values of the project: decentralization, utility, and openness.