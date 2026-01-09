#!/bin/bash
# Restore a checkpointed fortunate_v3.py process using CRIU
# Usage: ./restore-process.sh [checkpoint_directory]

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
echo "CRIU Process Restore Tool for fortunate_v3.py"
echo "================================================================"
echo ""

# Check if CRIU is available
if [ ! -f "$CRIU_BIN" ]; then
    echo -e "${RED}ERROR: CRIU binary not found at: $CRIU_BIN${NC}"
    echo "Set CRIU_BIN environment variable or install CRIU"
    exit 1
fi

# Check if checkpoint directory exists
if [ ! -d "$CHECKPOINT_DIR" ]; then
    echo -e "${RED}ERROR: Checkpoint directory not found: $CHECKPOINT_DIR${NC}"
    echo "Create a checkpoint first with: ./checkpoint-process.sh"
    exit 1
fi

echo "Checkpoint directory: $CHECKPOINT_DIR"

# Count process images
PROCESS_COUNT=$(ls -1 "$CHECKPOINT_DIR"/core-*.img 2>/dev/null | wc -l)
if [ "$PROCESS_COUNT" -eq 0 ]; then
    echo -e "${RED}ERROR: No checkpoint files found in $CHECKPOINT_DIR${NC}"
    exit 1
fi

echo "Process images found: $PROCESS_COUNT"
echo ""

# Show checkpoint info
CHECKPOINT_SIZE=$(du -sh "$CHECKPOINT_DIR" | cut -f1)
echo "Checkpoint size: $CHECKPOINT_SIZE"
echo ""

# Check if process is already running
EXISTING_PID=$(ps aux | grep "python.*fortunate_v3.py" | grep -v grep | awk '{print $2}' | head -1)
if [ -n "$EXISTING_PID" ]; then
    echo -e "${YELLOW}WARNING: fortunate_v3.py already running (PID: $EXISTING_PID)${NC}"
    read -p "Kill existing process and restore from checkpoint? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Killing process $EXISTING_PID..."
        kill -9 "$EXISTING_PID" 2>/dev/null || true
        sleep 2
    else
        echo "Aborted."
        exit 1
    fi
fi

# Restore the process
echo "Restoring process from checkpoint..."
echo "Command: sudo $CRIU_BIN restore -D $CHECKPOINT_DIR --shell-job -v4"
echo ""
echo -e "${YELLOW}Note: Terminal will show CRIU output, then processes will detach${NC}"
echo ""

if sudo "$CRIU_BIN" restore -D "$CHECKPOINT_DIR" --shell-job -v4; then
    echo ""
    echo -e "${GREEN}✓ Restore initiated${NC}"
    echo ""

    # Wait a moment for processes to start
    sleep 2

    # Verify process is running
    RESTORED_PID=$(ps aux | grep "python.*fortunate_v3.py" | grep -v grep | awk '{print $2}' | head -1)

    if [ -n "$RESTORED_PID" ]; then
        echo -e "${GREEN}✓ Process restored successfully!${NC}"
        echo ""
        echo "Main process PID: $RESTORED_PID"

        # Count workers
        WORKER_COUNT=$(ps --ppid "$RESTORED_PID" -o pid --no-headers 2>/dev/null | wc -l)
        echo "Worker processes: $WORKER_COUNT"
        echo ""

        # Show CPU usage
        echo "Process status:"
        ps -p "$RESTORED_PID" -o pid,ppid,pcpu,pmem,cmd --no-headers
        echo ""

        echo "================================================================"
        echo "Process restored and running!"
        echo "================================================================"
        echo ""
        echo "To monitor progress, check the log file in the working directory"
        echo "To create a new checkpoint, run: ./checkpoint-process.sh"
        echo ""
    else
        echo -e "${YELLOW}⚠ Restore completed but process not found${NC}"
        echo "Check 'ps aux | grep fortunate' to verify"
    fi
else
    echo ""
    echo -e "${RED}✗ Restore failed${NC}"
    echo ""
    echo "Common issues:"
    echo "  - Checkpoint corrupted or incomplete"
    echo "  - System state changed (different kernel, libraries, etc.)"
    echo "  - Insufficient permissions (need sudo)"
    echo "  - File descriptors or network connections not available"
    echo ""
    exit 1
fi
