# Setup Guide

This guide walks through compiling the prototype job manager and submitting a demo job.

## Prerequisites
- Rust toolchain (install with `rustup` from <https://rustup.rs>).

## Building
```bash
cd jobmanager
cargo build --release
```
This produces the `jobmanager` binary under `target/release/`.

## Prebuilt Binaries
Prebuilt archives are produced for Linux x86_64 and Apple Silicon whenever a `v*` tag is pushed. Visit the GitHub Releases page to download `devnet` and `jobmanager` without compiling.

## Posting a Job
From the `jobmanager` directory, run:
```bash
cargo run -- post "example job" 100
```
This stores a job in `jobs.json`.

## Listing Jobs
```bash
cargo run -- list
```
You should see the job ID, description, reward, and assignment status.

## Assigning and Completing
```bash
cargo run -- assign 1 my-worker
cargo run -- complete 1
```
These commands update `jobs.json` accordingly.

At this stage no full node exists yet. The job manager CLI acts as a lightweight demonstration of the workflow described in the [roadmap](ROADMAP.md).

## Mining a Dummy Block

The `devnet` crate now includes a `mine` command that demonstrates GPU usage during
block production. To run it:

```bash
cd ../devnet
cargo run -- mine
```
This command executes a simple compute shader to double some numbers and prints the result.

## Joining the Testnet

With the `p2p` crate you can run a node that communicates over TCP. Start one
node listening on a public port:

```bash
cargo run --manifest-path p2p/Cargo.toml --example node -- 8000
```

In another terminal (or on a different machine) connect to it:

```bash
cargo run --manifest-path p2p/Cargo.toml --example node -- 0 /ip4/ADDRESS/tcp/8000
```

Replace `ADDRESS` with the host running the first node. Nodes will exchange a
handshake and can then send training jobs. This opens the dev network to external
participants.
