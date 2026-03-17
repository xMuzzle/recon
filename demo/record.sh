#!/usr/bin/env bash
#
# record.sh — Build and record the recon demo GIF inside Docker.
#
# Usage: ./demo/record.sh
#
# Outputs: demo/out/tmux-demo.gif
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Building Docker image (compiles recon inside container) ==="
docker build -t recon-demo -f "$SCRIPT_DIR/Dockerfile" "$REPO_DIR"

echo "=== Recording demo ==="
mkdir -p "$SCRIPT_DIR/out"
docker run --rm --entrypoint bash -v "$SCRIPT_DIR/out:/output" recon-demo -c '
    set -euo pipefail

    # Ensure claude dirs exist
    mkdir -p ~/.claude/sessions ~/.claude/projects

    # Set up fake sessions
    /demo/demo.sh --setup &
    DEMO_PID=$!
    sleep 3

    # Record
    cd /demo && vhs tapes/tmux-demo.tape

    # Copy output
    cp /demo/tmux-demo.gif /output/ 2>/dev/null || true

    # Cleanup
    kill $DEMO_PID 2>/dev/null || true
'

echo "=== Done ==="
echo "Output: $SCRIPT_DIR/out/tmux-demo.gif"
