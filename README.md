![Saturn Devouring His Son](https://i.imgur.com/Ahvi7YS.gif)

## Build Instructions
```bash
sudo apt install clang
git clone https://github.com/cauchyteam/cauchy
cd cauchy
cargo build --release
```

## Running a Node
```bash
./target/release/cauchy
```
## Configuration
A custom configuration file `config.toml` may be added `$HOME\.cauchy\` directory. 

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
```


## RISCV Build Tools for C/C++ Scripts (Linux Only)
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

## RISCV Build Tools for Rust Scripts
The nightly build of Rust has support for the `riscv64gc-unknown-none-elf` target.  You can switch to nightlies and install this target like this:

```bash
rustup default nightly
rustup update
rustup target add riscv64gc-unknown-none-elf
```
After this, your Rust project must contain a `.cargo` directory with a `config` file containing the following:

```toml
[build]
target="riscv64gc-unknown-none-elf"
```