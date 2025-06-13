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

## Triggering a Release Build
To build and publish new archives, create and push a version tag:
```bash
git tag -a v0.1.0 -m "release v0.1.0"
git push origin v0.1.0
```
Pushing a tag that starts with `v` runs the `Build Releases` workflow and uploads the packaged binaries to GitHub.

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
