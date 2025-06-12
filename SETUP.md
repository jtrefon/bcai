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
