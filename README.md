![Saturn Devouring His Son](https://i.imgur.com/Ahvi7YS.gif)

## Build Instructions
```bash
sudo apt install clang
git clone https://github.com/cauchyteam/cauchy
cd cauchy
cargo build --release
```

## Configuration
Custom configuration can be done by adding `config.toml` file to the your `HOME_DIRECTORY\.geodesic\` directory. 

### Example

```toml
[NETWORK]
WORK_HEARTBEAT = 1_000_000_000
RECONCILE_HEARTBEAT = 5_000_000_000
RECONCILE_TIMEOUT = 4_000_000_000
SERVER_PORT = 8332
RPC_SERVER_PORT = 8333

[MINING]
N_MINING_THREADS = 2

[DEBUGGING]
TEST_TX_INTERVAL = 500_000
```
