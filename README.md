# Cauchy Ledger
Official Rust implementation of the Cauchy Ledger protocol.

[![Build Status](https://travis-ci.com/cauchyteam/cauchy.svg?branch=master)](https://travis-ci.com/cauchyteam/cauchy)

## Build Instructions
**Rust 1.34+**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Mongodb**
```bash
sudo apt install mongodb
```

**Cauchy Ledger**
```bash
git clone https://github.com/cauchyteam/cauchy
cd cauchy
cargo build --release
```

## Running a Node
```bash
./target/release/cauchy
```

## Configuration
Configuration is performed via `config.toml` in the `$HOME\.cauchy\` directory. 

### Example

```toml
[NETWORK]
WORK_HEARTBEAT_MS = 2_000
RECONCILE_HEARTBEAT_MS = 5_000
RECONCILE_TIMEOUT_MS = 5_000
SERVER_PORT = 8332
RPC_SERVER_PORT = 8333

[MINING]
N_MINING_THREADS = 2

[DEBUGGING]
TEST_TX_INTERVAL = 200
ARENA_VERBOSE = false
ENCODING_VERBOSE = false
DECODING_VERBOSE = false
PARSING_VERBOSE = false
DAEMON_VERBOSE = true
HEARTBEAT_VERBOSE = true
STAGE_VERBOSE = false
EGO_VERBOSE = true
RPC_VERBOSE = true
```


## RISC-V Build Tools for C/C++ Scripts (Linux Only)
Full instructions can be found [here](https://github.com/riscv/riscv-gnu-toolchain).  As an example, the gist for Ubuntu is

```bash
git clone --recursive https://github.com/riscv/riscv-gnu-toolchain
sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev libexpat-dev
mkdir /opt/riscv
./configure --prefix=/opt/riscv
make
```

Further, binaries should be compiled with the following GCC flags:
```bash
-nostdlib -s -Os
```

## RISC-V Build Tools for Rust Scripts
Your Rust project must contain a `.cargo` directory with a `config` file containing the following:

```toml
[build]
target="riscv64gc-unknown-none-elf"
```
