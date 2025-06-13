# Testnet Tutorial

This guide walks through joining the decentralized AI training testnet and posting your first job. It assumes you have the Rust toolchain installed as described in [SETUP.md](SETUP.md).

## 1. Build the binaries

```
# Build the job manager and devnet tools
cargo build --manifest-path jobmanager/Cargo.toml --release
cargo build --manifest-path devnet/Cargo.toml --release
```

These commands produce the `jobmanager` and `devnet` binaries under their respective `target/release` directories.

## 2. Run a testnet node

Start a node listening on TCP port `8000`:

```
cargo run --manifest-path p2p/Cargo.toml --example node -- 8000
```

Leave this process running. It forms a small network and waits for peers.

To connect another node from a different terminal or machine, run:

```
cargo run --manifest-path p2p/Cargo.toml --example node -- 0 /ip4/ADDRESS/tcp/8000
```

Replace `ADDRESS` with the IP address of the first node.

## 3. Post a training job

In a new terminal, post a job using the job manager CLI:

```
cargo run --manifest-path jobmanager/Cargo.toml -- post "train digits" 100
```

This stores the job in `jobs.json` and prints the assigned job ID.

## 4. Assign and complete

Assign a worker and mark the job complete when finished:

```
cargo run --manifest-path jobmanager/Cargo.toml -- assign JOB_ID worker1
cargo run --manifest-path jobmanager/Cargo.toml -- complete JOB_ID
```

Rewards are tracked in the ledger maintained by the devnet.

## 5. View jobs in the dashboard

Run the simple HTTP dashboard to see posted jobs:

```
cargo run --manifest-path dashboard/Cargo.toml --
```

Visit `http://127.0.0.1:8000/jobs` in your browser to view the job list.

With these steps you can join the testnet, post jobs, and verify activity through the dashboard.
