# Implementation Planning

This document breaks down roadmap milestones into actionable tasks for early development.

## Milestone 2: Prototype Development (Q1 2026)
- [ ] Decide on the base platform (Substrate vs. EVM fork)
- [ ] Initialize a development network with staking and basic token logic
- [ ] Implement a minimal JobManager contract/pallet for posting tasks
- [ ] Spike GPU integration by executing a dummy ML task during block production
- [ ] Create a CLI/SDK prototype for submitting jobs
- [ ] Write setup docs for running a node and submitting a job

## Milestone 3: Testnet Alpha (Q2 2026)
- [ ] Implement a simple Proof-of-Useful-Work consensus algorithm
- [ ] Develop trainer and evaluator node roles
- [ ] Add basic P2P networking between nodes
- [ ] Deploy early smart contracts on the devnet
- [ ] Launch an internal dashboard/explorer for jobs
- [ ] Run a closed testnet with example ML tasks

## Milestone 4: Testnet Beta (Q3 2026)
- [ ] Add slashing and reputation tracking to penalize misbehavior
- [ ] Integrate real model training (e.g., MNIST) in block production
- [ ] Open the testnet to outside node operators
- [ ] Provide tutorials for joining the testnet and posting jobs
- [ ] Hold a community challenge using testnet tokens

Additional milestones are outlined in `ROADMAP.md`. This checklist focuses on the near-term steps to bootstrap development and move toward a functional testnet.
