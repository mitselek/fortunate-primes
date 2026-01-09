#!/bin/bash
# Checkpoint a running fortunate_v3.py process using CRIU
# Usage: sudo ./checkpoint-process.sh [checkpoint_directory]
#
# Requires: CRIU v4.2+ (see CRIU_SETUP.md for installation)

set -e

# Configuration
CRIU_BIN="${CRIU_BIN:-/tmp/criu-4.2/criu/criu}"
DEFAULT_CHECKPOINT_DIR="$HOME/Documents/github/mitselek/projects/fortunate-primes/fortunate_checkpoint"
CHECKPOINT_DIR="${1:-$DEFAULT_CHECKPOINT_DIR}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "================================================================"
echo "CRIU Process Checkpoint Tool for fortunate_v3.py"
echo "================================================================"
echo ""

# Check if running with sudo
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}ERROR: This script must be run with sudo${NC}"
    echo "Usage: sudo ./checkpoint-process.sh [checkpoint_directory]"
    exit 1
fi

# Check if CRIU is available
if [ ! -f "$CRIU_BIN" ]; then
    echo -e "${RED}ERROR: CRIU binary not found at: $CRIU_BIN${NC}"
    echo ""
    echo "Install CRIU following instructions in CRIU_SETUP.md:"
    echo "  1. Install dependencies: sudo apt install build-essential libprotobuf-dev ..."
    echo "  2. Download and build: cd /tmp && wget https://github.com/checkpoint-restore/criu/archive/v4.2/criu-4.2.tar.gz ..."
    echo "  3. Set CRIU_BIN: export CRIU_BIN=/tmp/criu-4.2/criu/criu"
    echo ""
    echo "See CRIU_SETUP.md for complete instructions."
    exit 1
fi

# Find the main fortunate_v3.py process
echo "Searching for fortunate_v3.py process..."
MAIN_PID=$(ps aux | grep "python.*fortunate_v3.py" | grep -v grep | awk '{print $2}' | head -1)

if [ -z "$MAIN_PID" ]; then
    echo -e "${RED}ERROR: No fortunate_v3.py process found${NC}"
    echo "Start the computation first with: python fortunate_v3.py <start> <end>"
    exit 1
fi

echo -e "${GREEN}Found process PID: $MAIN_PID${NC}"

# Get process details
PS_INFO=$(ps -p "$MAIN_PID" -o pid,ppid,cmd --no-headers)
echo "Process info: $PS_INFO"
echo ""

# Count worker processes
WORKER_COUNT=$(ps --ppid "$MAIN_PID" -o pid --no-headers | wc -l)
echo "Worker processes: $WORKER_COUNT"
echo ""

# Create checkpoint directory
echo "Checkpoint directory: $CHECKPOINT_DIR"
if [ -d "$CHECKPOINT_DIR" ]; then
    echo -e "${YELLOW}WARNING: Checkpoint directory exists${NC}"
    read -p "Overwrite existing checkpoint? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
    sudo rm -rf "$CHECKPOINT_DIR"
fi

sudo mkdir -p "$CHECKPOINT_DIR"
echo ""

# Checkpoint the process
echo "Starting checkpoint (this will freeze the process briefly)..."
echo "Command: sudo $CRIU_BIN dump -t $MAIN_PID -D $CHECKPOINT_DIR --shell-job -v4"
echo ""

if sudo "$CRIU_BIN" dump -t "$MAIN_PID" -D "$CHECKPOINT_DIR" --shell-job -v4 2>&1 | tail -50; then
    echo ""
    echo -e "${GREEN}✓ Checkpoint completed successfully!${NC}"
    echo ""

    # Show checkpoint info
    echo "Checkpoint contents:"
    ls -lh "$CHECKPOINT_DIR" | head -10
    echo ""

    CHECKPOINT_SIZE=$(du -sh "$CHECKPOINT_DIR" | cut -f1)
    echo "Total checkpoint size: $CHECKPOINT_SIZE"
    echo ""

    # Count process images
    PROCESS_COUNT=$(ls -1 "$CHECKPOINT_DIR"/core-*.img 2>/dev/null | wc -l)
    echo "Process images saved: $PROCESS_COUNT"
    echo ""

    echo "================================================================"
    echo "Checkpoint saved successfully!"
    echo "================================================================"
    echo ""
    echo "To restore the process later, use:"
    echo "  sudo $CRIU_BIN restore -D $CHECKPOINT_DIR --shell-job"
    echo ""
    echo "Or use the restore-process.sh script:"
    echo "  sudo ./restore-process.sh $CHECKPOINT_DIR"
    echo ""
else
    echo ""
    echo -e "${RED}✗ Checkpoint failed${NC}"
    echo ""
    echo "Common issues:"
    echo "  - Process not running as expected"
    echo "  - Insufficient permissions (need sudo)"
    echo "  - CRIU not compatible with kernel"
    echo ""
    exit 1
fi
