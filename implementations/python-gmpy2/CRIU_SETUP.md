# CRIU Installation Guide

CRIU (Checkpoint/Restore In Userspace) enables saving and restoring running processes to/from disk.

## Prerequisites

Ubuntu 24.04 or later with kernel 3.11+ (check: `uname -r`)

## Installation Steps

### 1. Install Build Dependencies

```bash
sudo apt install -y \
    build-essential \
    libprotobuf-dev \
    libprotobuf-c-dev \
    protobuf-c-compiler \
    protobuf-compiler \
    python3-protobuf \
    pkg-config \
    uuid-dev \
    libbsd-dev \
    libcap-dev \
    libnet1-dev \
    libnl-3-dev \
    libnl-route-3-dev \
    libaio-dev
```

### 2. Download and Build CRIU

```bash
cd /tmp
wget https://github.com/checkpoint-restore/criu/archive/v4.2/criu-4.2.tar.gz
tar xzf criu-4.2.tar.gz
cd criu-4.2
make -j$(nproc)
```

Build time: ~30 seconds on modern hardware.

### 3. Verify Build

```bash
sudo ./criu/criu check
```

Expected output: `Looks good.`

### 4. (Optional) System-Wide Installation

```bash
cd /tmp/criu-4.2
sudo make install
```

**Note**: Man page installation may fail (needs `asciidoc`), but the binary installs successfully to `/usr/local/sbin/criu`.

### 5. Configure Environment Variable

Add to your `~/.bashrc` or `~/.zshrc`:

```bash
# CRIU binary location (choose one)
export CRIU_BIN=/tmp/criu-4.2/criu/criu          # Build directory (temporary)
# OR
export CRIU_BIN=/usr/local/sbin/criu              # System-wide install
```

Then reload: `source ~/.bashrc`

## Usage with fortunate_v3.py

### Checkpoint Running Process

```bash
cd projects/fortunate-primes/implementations/python-gmpy2
./checkpoint-process.sh
```

### Restore Process

```bash
./restore-process.sh
```

## Troubleshooting

### "CRIU binary not found"

- Set `CRIU_BIN` environment variable pointing to CRIU executable
- Or pass custom path: `CRIU_BIN=/path/to/criu ./checkpoint-process.sh`

### "the PARI stack overflows"

Not related to CRIU - this is PARI/GP memory issue. Increase with `default(parisize, 2000000000)`.

### Checkpoint fails with permission errors

- Ensure running with `sudo` (scripts handle this automatically)
- Check kernel capabilities: `sudo ./criu/criu check --all`

### Restore fails

- Ensure system state is similar (same kernel, libraries)
- Check dmesg: `dmesg | tail -50`
- Try verbose mode: Edit script to use `-vvv` instead of `-v4`

## Technical Details

**CRIU v4.2 "CRIUTIBILITY"**

- Kernel requirement: 3.11+
- Checkpoint size: ~7.5MB per GB of process memory
- Typical checkpoint time: <1 second
- Typical restore time: 1-3 seconds

**What's Saved**:

- All process memory (heap, stack, libraries)
- File descriptors and open files
- Environment variables
- Shared memory (multiprocessing queues)
- Thread states
- Network connections (TCP/UDP)

**What's NOT Saved**:

- External network connections from other machines
- Hardware state changes
- Time (processes resume with current system time)

## References

- CRIU Documentation: https://criu.org/
- GitHub Repository: https://github.com/checkpoint-restore/criu
- Installation Guide: https://criu.org/Installation
