# Implementation Planning

This document breaks down roadmap milestones into actionable tasks for early development.

## Milestone 2: Prototype Development (Q1 2026)
- [x] Design and implement the custom VM runtime (initial skeleton in `runtime` crate)
- [x] Initialize a development network with staking and basic token logic
- [x] Implement a minimal JobManager contract/pallet for posting tasks
- [x] Spike GPU integration by executing a dummy ML task during block production
- [x] Create a CLI/SDK prototype for submitting jobs (`jobmanager` crate in this repo)
- [x] Write setup docs for running a node and submitting a job (see `SETUP.md`)

## Milestone 3: Testnet Alpha (Q2 2026)
- [x] Implement a simple Proof-of-Useful-Work consensus algorithm (see `runtime/src/pouw.rs`)
- [x] Develop trainer and evaluator node roles
- [x] Add basic P2P networking between nodes
- [x] Deploy early runtime modules on the devnet
- [x] Launch an internal dashboard/explorer for jobs
- [ ] Run a closed testnet with example ML tasks

## Milestone 4: Testnet Beta (Q3 2026)
- [ ] Add slashing and reputation tracking to penalize misbehavior
- [ ] Integrate real model training (e.g., MNIST) in block production
- [ ] Open the testnet to outside node operators
- [ ] Provide tutorials for joining the testnet and posting jobs
- [ ] Hold a community challenge using testnet tokens

Additional milestones are outlined in `ROADMAP.md`. This checklist focuses on the near-term steps to bootstrap development and move toward a functional testnet.
