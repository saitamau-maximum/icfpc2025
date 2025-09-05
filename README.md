# ICFPC 2025

## CLI

### Build

```bash
cargo build --release
```

### Usage

```bash
# Select a problem
./target/release/aedificium select probatio

# Select a problem from stdin
./target/release/aedificium select < problem.txt

# Explore multiple plans
./target/release/aedificium explore "1,2,3"

# Explore multiple plans from stdin
./target/release/aedificium explore < plans.txt

# Submit a map guess
./target/release/aedificium guess '{"rooms":[1,2],"startingRoom":1,"connections":[]}'

# Submit a map guess from stdin
./target/release/aedificium guess < map.json
```
