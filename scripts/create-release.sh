#!/bin/bash

# Script to create a new release for BCAI
# Usage: ./scripts/create-release.sh <version>
# Example: ./scripts/create-release.sh v0.1.0

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.1.0"
    exit 1
fi

# Validate version format
if [[ ! $VERSION =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
    echo "Error: Version must be in format v0.0.0 or v0.0.0-alpha"
    echo "Example: v0.1.0, v1.2.3, v0.1.0-beta"
    exit 1
fi

echo "Creating release $VERSION..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "Error: Not in a git repository"
    exit 1
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean. Please commit or stash changes."
    git status --short
    exit 1
fi

# Make sure we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "Warning: Not on main branch (currently on $CURRENT_BRANCH)"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted"
        exit 1
    fi
fi

# Check if tag already exists
if git tag -l | grep -q "^$VERSION$"; then
    echo "Error: Tag $VERSION already exists"
    exit 1
fi

# Pull latest changes
echo "Pulling latest changes..."
git pull origin main

# Run tests to make sure everything works
echo "Running tests..."
cargo test --manifest-path runtime/Cargo.toml
cargo test --manifest-path jobmanager/Cargo.toml

# Create and push tag
echo "Creating tag $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION"

echo "Pushing tag to GitHub..."
git push origin "$VERSION"

echo ""
echo "✅ Release $VERSION created successfully!"
echo ""
echo "The GitHub Actions workflow will now:"
echo "  1. Build binaries for Linux, Windows, and macOS"
echo "  2. Create release packages"
echo "  3. Publish the release on GitHub"
echo ""
echo "Check the progress at: https://github.com/jtrefon/bcai/actions"
echo "Release will be available at: https://github.com/jtrefon/bcai/releases" 