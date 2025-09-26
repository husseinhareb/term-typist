#!/usr/bin/env bash
set -euo pipefail

# Script to install system dependencies required to build and run term-typist
# Run with: sudo ./scripts/install-deps.sh

apt-get update
apt-get install -y pkg-config libasound2-dev

echo "System dependencies installed: pkg-config, libasound2-dev" 
