# 🚀 Limitless Distributed Storage – Implementation Plan

> **Objective:** Remove legacy 3-TB hard limit, support arbitrarily large datasets, user-configurable redundancy, and up-front price quoting in BCAI tokens.

---
## 1  Background
Legacy PoC code capped transfers at 3 TB. Phase-3 training requires petabyte-scale data. We will extend the existing chunk-based transfer layer without violating the ≤ 100 LOC rule for production files.

---
## 2  Architecture Overview
```mermaid
flowchart TD
  CLI["CLI store --copies N"] -->|split_into_chunks| CM(ChunkManager)
  CM -->|push(chunk)| PM(P2P PeerManager)
  CM -->|quote()| PE(Pricing Engine)
  PM --> CM
```
* **RedundancyPolicy**: `{ copies: u8, geo_spread: bool }` (default copies = 1).
* **Pricing Engine**: pure function → deterministic unit tests.
* **Auto-heal**: missing replica triggers `replicate_missing_chunks()`.

---
## 3  Work Breakdown Structure
| Step | Description | LOC Δ |
|---:|---|---:|
| 1 | Delete every "3 TB" constant / comment / test | − |
| 2 | Add `RedundancyPolicy`; embed in metadata | +20 |
| 3 | Implement `pricing.rs` + CLI quote path | +60 |
| 4 | Add `redundancy_copies` to ChunkManager | +40 |
| 5 | Latency-aware peer selection | +15 |
| 6 | New CLI subcommands (`store`, `quote-only`, `rebalance`) | +80 |
| 7 | Docs + integration tests (petabyte dataset) | − |

---
## 4  Acceptance Criteria
1. No file-size limit in production code.
2. CLI shows accurate cost before upload.
3. Redundancy ≥ 1, default = 1.
4. Automatic re-replication on host loss.
5. All prod files remain ≤ 100 LOC.
6. `cargo build && cargo test` green.

---
## 5  Timeline (tentative)
| Day | Milestone |
|---|---|
| 0 | Step 1 merged |
| 2 | Step 2 + 3 merged |
| 5 | Step 4 merged |
| 6 | Step 5 merged |
| 7 | Step 6 merged |
| 8 | Step 7 docs release |

---
## 6  Next Action
Create branch `feature/limitless-storage-step1` and remove all 3-TB references.

---
© 2025 BCAI Project – All rights reserved. 