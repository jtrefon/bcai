# Release Process

This document explains how to create releases for the BCAI project and troubleshoot common issues.

## Overview

The BCAI project uses GitHub Actions to automatically build and publish releases across multiple platforms:

- **Linux x86_64** (`ubuntu-latest`)
- **Windows x86_64** (`windows-latest`) 
- **macOS ARM64** (`macos-14` - Apple Silicon)
- **macOS x86_64** (`macos-13` - Intel)

Each release includes all four binaries:
- `devnet` - Development network node
- `jobmanager` - Job management CLI
- `keygen` - Key generation utility  
- `dashboard` - Network dashboard

## Creating a Release

### Method 1: Using the Helper Script (Recommended)

```bash
# Make the script executable (first time only)
chmod +x scripts/create-release.sh

# Create a release
./scripts/create-release.sh v0.1.0
```

The script will:
1. Validate the version format
2. Check that your working directory is clean
3. Run tests to ensure everything works
4. Create and push a git tag
5. Trigger the GitHub Actions workflow

### Method 2: Manual Process

```bash
# Ensure you're on main branch with clean working directory
git checkout main
git pull origin main

# Run tests
cargo test --manifest-path runtime/Cargo.toml
cargo test --manifest-path jobmanager/Cargo.toml

# Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

### Method 3: Manual GitHub Workflow Trigger

You can also trigger a release manually through GitHub:

1. Go to the **Actions** tab in your GitHub repository
2. Select **Build and Release** workflow
3. Click **Run workflow**
4. Enter the tag name (e.g., `v0.1.0`)
5. Click **Run workflow**

## Testing the Release Pipeline

Before creating an actual release, you can test the build process:

1. Go to **Actions** tab → **Test Release Build**
2. Click **Run workflow**
3. Select the platform to test
4. Click **Run workflow**

This will build binaries for the selected platform without creating a release.

## Why You Might Not See Releases

### Common Issues and Solutions

#### 1. **No Git Tags**
**Issue**: Release workflow only triggers on tags starting with `v*`
```bash
# Check if you have any tags
git tag -l

# If empty, create your first release
./scripts/create-release.sh v0.1.0
```

#### 2. **Workflow Permissions**
**Issue**: GitHub Actions might not have permission to create releases
**Solution**: Check repository settings:
- Go to **Settings** → **Actions** → **General**
- Under "Workflow permissions", ensure "Read and write permissions" is selected
- Check "Allow GitHub Actions to create and approve pull requests"

#### 3. **Build Failures**
**Issue**: One or more platform builds are failing
**Solution**: Check the Actions tab for error details:
- Click on the failed workflow run
- Expand the failed step to see error details
- Common issues:
  - Missing dependencies
  - Platform-specific code issues
  - Cargo.toml configuration problems

#### 4. **Binary Not Found Errors**
**Issue**: Workflow can't find the compiled binaries
**Solution**: Ensure all crates have proper `[[bin]]` sections in their `Cargo.toml`:
```toml
[[bin]]
name = "devnet"
path = "src/main.rs"
```

#### 5. **Release Not Published**
**Issue**: Build succeeds but release doesn't appear
**Solution**: Check the release job logs:
- Release job depends on all build jobs completing successfully
- Verify `GITHUB_TOKEN` has proper permissions
- Check for API rate limiting issues

## Troubleshooting Commands

```bash
# Check current git status
git status

# List all tags
git tag -l

# Check remote repository
git remote -v

# Test build locally
cargo build --release --manifest-path devnet/Cargo.toml
cargo build --release --manifest-path jobmanager/Cargo.toml
cargo build --release --manifest-path keygen/Cargo.toml
cargo build --release --manifest-path dashboard/Cargo.toml

# Test binaries work
./target/release/devnet --help
./target/release/jobmanager --help
./target/release/keygen --help
./target/release/dashboard --help
```

## Workflow Files

The release process uses these GitHub Actions workflows:

- **`.github/workflows/release.yml`**: Main release workflow (triggered by tags)
- **`.github/workflows/ci.yml`**: Continuous integration (runs on every push/PR)
- **`.github/workflows/test-release.yml`**: Manual testing workflow

## Release Versioning

Follow semantic versioning (SemVer):
- `v1.0.0` - Major release
- `v0.1.0` - Minor release  
- `v0.0.1` - Patch release
- `v0.1.0-alpha` - Pre-release

## After Creating a Release

1. **Verify the release**: Check https://github.com/jtrefon/bcai/releases
2. **Test downloads**: Download and test binaries on different platforms
3. **Update documentation**: Update README or changelog as needed
4. **Announce**: Notify users through appropriate channels

## Monitoring

- **GitHub Actions**: https://github.com/jtrefon/bcai/actions
- **Releases**: https://github.com/jtrefon/bcai/releases
- **Issues**: Report release-related issues in the repository

## Advanced Configuration

### Adding New Platforms

To add support for additional platforms, edit `.github/workflows/release.yml`:

```yaml
- os: ubuntu-latest
  target: aarch64-unknown-linux-gnu
  artifact: linux-aarch64
  binary_ext: ""
```

### Custom Release Notes

The workflow automatically generates release notes, but you can customize them by editing the `body` section in the release workflow.

### Artifacts Retention

Build artifacts are kept for 5 days by default. Adjust the `retention-days` in the workflow if needed. 