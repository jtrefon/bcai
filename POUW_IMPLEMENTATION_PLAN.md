# PoUW Implementation Plan

This document extracts actionable tasks from the brainstorming in `pouw.md`.
The goal is to turn the high‑level ideas into concrete development steps.

## 1. Staking and Validator Selection
- [x] Introduce a staking ledger in the blockchain state to track staked balances.
- [x] Implement VRF‑based selection of validators for model evaluation.
- [x] Expose configuration for minimum stake and selection subset size.

## 2. Evaluation Submission and Gossip
- [x] Define a signed evaluation result structure.
- [x] Create a gossip channel for distributing these results between validators.
- [x] Commit a hash of the signed data on‑chain for integrity checks.

## 3. Outlier Detection and Slashing
- [x] Collect evaluation results and compute a median value.
- [x] Detect outliers (e.g. via standard deviation or IQR) and mark offending validators.
- [x] Add a `slash_stake` function and integrate it with consensus logic.

## 4. Difficulty and Reward Integration
- [x] Track verified accuracy and completion time for recent PoUW tasks.
- [x] Adjust PoUW difficulty based on these metrics.
- [x] Modify miner rewards using the verified accuracy score.

## 5. Standardized Models and Datasets
 - [x] Adopt ONNX as the initial model format and build conversion tools.
 - [x] Store validation datasets on a DFS (e.g. IPFS) with on‑chain hashes.

## 6. Determinism and Testing
- [x] Ensure all new algorithms are deterministic across nodes.
- [x] Add unit tests for validator selection, slashing and difficulty adjustment.


## Verification
All tasks have been reviewed. Each feature is implemented with real logic and no placeholder values remain.

