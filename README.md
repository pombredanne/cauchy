![Saturn Devouring His Son](https://i.imgur.com/Ahvi7YS.gif)

## Build Instructions
```bash
sudo apt install clang
git clone https://github.com/cauchyteam/cauchy
cd cauchy
cargo build --release
```

## Running a Node
After build completed
```bash
./target/release/cauchy
```
## Configuration
A custom configuration file `config.toml` may be added `HOME_DIRECTORY\.cauchy\` directory. 

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
```
